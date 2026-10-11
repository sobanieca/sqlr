use super::MAX_TABLE_COLUMN_WIDTH;
use crate::colors;
use crate::connection;
use crate::connectors::{self, Error, QueryResult};
use crate::inspect::{self, Node};
use crate::logger;
use crate::storage::Storage;
use crate::table::Table;
use crate::value;
use std::time::Instant;

pub struct Options {
    pub query: Option<String>,
    pub input_variables: Vec<String>,
    pub output_file: Option<String>,
    pub name: Option<String>,
    pub table: bool,
    pub compact: bool,
    pub connection_type: Option<String>,
    pub connection_string: Option<String>,
    pub ignore_input_validation: bool,
    pub is_global: bool,
}

fn parse_input_variables(items: &[String]) -> Result<Vec<(String, String)>, String> {
    items
        .iter()
        .map(|item| {
            let (key, value) = item.split_once(':').ok_or_else(|| {
                format!("Invalid input variable format: \"{item}\". Expected \"key: value\".")
            })?;
            Ok((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

/// Names of `{{name}}` placeholders, without duplicates.
fn placeholders(query: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let mut rest = query;
    while let Some(start) = rest.find("{{") {
        let after = &rest[start + 2..];
        let name_len = after.find('}').unwrap_or(after.len());
        if name_len > 0 && after[name_len..].starts_with("}}") {
            let name = after[..name_len].to_string();
            if !names.contains(&name) {
                names.push(name);
            }
            rest = &after[name_len + 2..];
        } else {
            rest = &rest[start + 1..];
        }
    }
    names
}

fn replace_variables(
    query: &str,
    variables: &[(String, String)],
    ignore_input_validation: bool,
) -> Result<String, String> {
    let mut result = query.to_string();
    for (key, value) in variables {
        result = result.replace(&format!("{{{{{key}}}}}"), value);
    }
    if !ignore_input_validation {
        let missing = placeholders(&result);
        if !missing.is_empty() {
            return Err(format!(
                "Missing input variables: {}. Use -i \"name: value\" to provide them.",
                missing.join(", ")
            ));
        }
    }
    Ok(result)
}

fn preview(query: &str) -> String {
    let lines: Vec<&str> = query.split('\n').collect();
    if lines.len() > 10 {
        let mut shown = lines[..10].to_vec();
        shown.push("...");
        shown.join("\n")
    } else {
        query.to_string()
    }
}

pub fn execute(options: Options) -> Result<(), String> {
    let storage = Storage::load()?;
    let mut name = options.name;
    if name.is_none() && options.connection_string.is_none() {
        name = Some(connection::select_name(&storage, options.is_global)?);
    }

    let mut connection_type = options.connection_type;
    let mut connection_string = options.connection_string;
    if let Some(name) = name {
        let connection = connection::get(&storage, &name, options.is_global)?;
        logger::info(connection::format_connection_name(&connection));
        connection_type = Some(connection.connection_type);
        connection_string = Some(connection.connection_string);
    }

    let Some(query_arg) = options.query else {
        return Err("No SQL query provided".to_string());
    };
    let is_file = std::fs::metadata(&query_arg).is_ok_and(|m| m.is_file());
    let mut query = if is_file {
        std::fs::read_to_string(&query_arg)
            .map_err(|err| format!("Failed to read {query_arg}: {err}"))?
    } else {
        query_arg
    };

    let variables = parse_input_variables(&options.input_variables)?;
    query = replace_variables(&query, &variables, options.ignore_input_validation)?;

    if is_file {
        logger::info(format!(
            "{}\n{}",
            colors::gray("SQL to execute:"),
            preview(&query)
        ));
    }

    let connector = connection_type.as_deref().and_then(connectors::find);
    let started = Instant::now();
    let result = match (connector, connection_string) {
        (Some(connector), Some(connection_string)) => {
            (connector.query)(&connection_string, &query)
        }
        _ => Err(Error::Other("Missing connection type or connection string".to_string())),
    };
    let elapsed = started.elapsed().as_millis();

    match result {
        Ok(result) => {
            logger::info(format!("{} {elapsed}ms", colors::gray("Query executed in:")));
            logger::info(format!(
                "{} {}",
                colors::gray("Rows affected:"),
                value::to_js_string(&result.rows_affected)
            ));
            show_result(&result, options.output_file, options.table, options.compact)
        }
        Err(Error::Database(message)) => Err(message),
        Err(Error::Other(message)) => {
            logger::debug(message);
            Err("Error occurred when executing query against database. Ensure that query or connection string is valid. Only simple queries are supported. Use --debug option for details.".to_string())
        }
    }
}

fn show_result(
    result: &QueryResult,
    output_file: Option<String>,
    table: bool,
    compact: bool,
) -> Result<(), String> {
    let Some(first) = result.rows.first() else {
        return Ok(());
    };

    if let Some(output_file) = output_file {
        std::fs::write(&output_file, value::rows_to_json(&result.rows))
            .map_err(|err| format!("Failed to write {output_file}: {err}"))?;
        logger::info(format!("Results saved to {output_file}"));
        return Ok(());
    }

    if table {
        let header = first.keys().map(|key| colors::bright_green(key)).collect();
        let body = result
            .rows
            .iter()
            .map(|row| row.values().map(value::to_display_string).collect())
            .collect();
        logger::info(
            Table::new()
                .header(header)
                .body(body)
                .max_col_width(MAX_TABLE_COLUMN_WIDTH)
                .padding(1)
                .indent(2)
                .border(true)
                .to_string(),
        );
        return Ok(());
    }

    if compact {
        logger::info(colors::bright_green(
            &first.keys().cloned().collect::<Vec<_>>().join(","),
        ));
        for row in &result.rows {
            logger::info(
                row.values()
                    .map(value::to_display_string)
                    .collect::<Vec<_>>()
                    .join(","),
            );
        }
        return Ok(());
    }

    let rows = Node::Array(result.rows.iter().map(value::object_node).collect());
    logger::info(inspect::inspect(&rows, inspect::Options {
                colors: colors::enabled(),
                ..inspect::Options::verbose()
            }));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_variables_literally() {
        let variables = parse_input_variables(&["col: 1".into(), "alias: a$&b".into()]).unwrap();
        assert_eq!(
            replace_variables("SELECT {{col}} as {{alias}}", &variables, false).unwrap(),
            "SELECT 1 as a$&b"
        );
        assert_eq!(
            replace_variables("SELECT '{{x}}', {{y}}, {{x}}", &[], false).unwrap_err(),
            "Missing input variables: x, y. Use -i \"name: value\" to provide them."
        );
        assert_eq!(
            replace_variables("SELECT '{{x}}'", &[], true).unwrap(),
            "SELECT '{{x}}'"
        );
        assert!(parse_input_variables(&["novalue".into()]).is_err());
    }

    #[test]
    fn finds_placeholders_like_the_regex() {
        assert_eq!(placeholders("{{a}} {{}} {{b}}} {{a}} {{c"), ["a", "b"]);
        assert_eq!(placeholders("{{{x}}}"), ["{x"]);
    }

    #[test]
    fn previews_the_first_ten_lines() {
        let query = (1..=12).map(|i| i.to_string()).collect::<Vec<_>>().join("\n");
        assert_eq!(preview(&query), "1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n...");
        assert_eq!(preview("SELECT 1;\n"), "SELECT 1;\n");
    }
}
