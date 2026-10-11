use super::MAX_TABLE_COLUMN_WIDTH;
use crate::colors;
use crate::connection;
use crate::connectors::{self, Column, Error, Table as DbTable};
use crate::inspect::{self, Node};
use crate::logger;
use crate::storage::Storage;
use crate::table::Table;
use std::cmp::Ordering;

pub struct Options {
    pub name: Option<String>,
    pub table: bool,
    pub compact: bool,
    pub tables_only: bool,
    pub connection_type: Option<String>,
    pub filter: Option<String>,
    pub connection_string: Option<String>,
    pub is_global: bool,
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

    let connector = connection_type.as_deref().and_then(connectors::find);
    let tables = match (connector, connection_string) {
        (Some(connector), Some(connection_string)) => (connector.get_tables)(&connection_string),
        _ => Err(Error::Other(
            "Missing connection type or connection string".to_string(),
        )),
    };
    let mut tables = tables.map_err(|err| {
        logger::debug(match err {
            Error::Database(message) | Error::Other(message) => message,
        });
        "Error occurred when executing query against database. Ensure connection string is valid. Use --debug option for details.".to_string()
    })?;

    tables.sort_by(|a, b| locale_compare(&a.schema, &b.schema).then(locale_compare(&a.name, &b.name)));
    if let Some(filter) = &options.filter {
        tables.retain(|table| format!("{}.{}", table.schema, table.name).contains(filter.as_str()));
    }

    logger::info("Database schema:");
    if options.tables_only {
        for table in &tables {
            logger::info(table_name(table));
        }
    } else if options.compact {
        show_compact(&tables);
    } else if options.table {
        show_tables(&tables);
    } else {
        let nodes = Node::Array(tables.iter().map(table_node).collect());
        logger::info(inspect::inspect(&nodes, inspect::Options::verbose()));
    }
    Ok(())
}

fn table_node(table: &DbTable) -> Node {
    Node::Object {
        class: None,
        entries: vec![
            ("schema".to_string(), Node::String(table.schema.clone())),
            ("name".to_string(), Node::String(table.name.clone())),
            (
                "columns".to_string(),
                Node::Array(table.columns.iter().map(column_node).collect()),
            ),
        ],
    }
}

fn column_node(column: &Column) -> Node {
    let mut entries = vec![
        ("name".to_string(), Node::String(column.name.clone())),
        ("type".to_string(), Node::String(column.data_type.clone())),
        ("nullable".to_string(), Node::Bool(column.nullable)),
    ];
    if let Some(relation) = column.relation.as_ref().filter(|r| !r.is_empty()) {
        entries.push(("relation".to_string(), Node::String(relation.clone())));
    }
    Node::Object {
        class: None,
        entries,
    }
}

fn show_tables(tables: &[DbTable]) {
    for table in tables {
        logger::info(
            Table::new()
                .header(vec![table_name(table)])
                .body(
                    table
                        .columns
                        .iter()
                        .map(|column| vec![column_description(column)])
                        .collect(),
                )
                .max_col_width(MAX_TABLE_COLUMN_WIDTH)
                .padding(1)
                .indent(2)
                .border(true)
                .to_string(),
        );
    }
}

fn show_compact(tables: &[DbTable]) {
    for table in tables {
        let columns: Vec<String> = table.columns.iter().map(column_description).collect();
        logger::info(format!("{} [ {} ]", table_name(table), columns.join(", ")));
    }
}

fn table_name(table: &DbTable) -> String {
    format!(
        "{}.{}",
        colors::bright_green(&table.schema),
        colors::bright_green(&table.name)
    )
}

fn column_description(column: &Column) -> String {
    match column.relation.as_ref().filter(|r| !r.is_empty()) {
        Some(relation) => format!(
            "{} -> \"{}\"",
            colors::bright_yellow(&column.name),
            colors::bright_blue(relation)
        ),
        None => format!(
            "{}{} {}",
            column.name,
            if column.nullable {
                colors::bright_red("?")
            } else {
                String::new()
            },
            colors::gray(&column.data_type)
        ),
    }
}

/// Approximation of `String.prototype.localeCompare` (ICU root collation):
/// punctuation sorts before digits and letters, letters compare case
/// insensitively first and lowercase sorts before uppercase.
fn locale_compare(a: &str, b: &str) -> Ordering {
    const PUNCTUATION: &str = "\t\n\r _-,;:!?.'\"()[]{}@*/\\&#%`^+<=>|~$";
    fn primary(c: char) -> (u8, usize, char) {
        if c.is_alphabetic() {
            (2, 0, c.to_lowercase().next().unwrap_or(c))
        } else if c.is_numeric() {
            (1, 0, c)
        } else {
            (0, PUNCTUATION.find(c).unwrap_or(PUNCTUATION.len()), c)
        }
    }
    let primary_order = a.chars().map(primary).cmp(b.chars().map(primary));
    primary_order
        .then_with(|| {
            a.chars()
                .map(|c| c.is_uppercase())
                .cmp(b.chars().map(|c| c.is_uppercase()))
        })
        .then_with(|| a.cmp(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorts_like_locale_compare() {
        let mut names = vec![
            "country_language", "City", "country", "_tmp", "2fa", "city", "countryflag", "a-b",
            "a_b", "ab", "A", "a", "b1", "b10", "b2",
        ];
        names.sort_by(|a, b| locale_compare(a, b));
        assert_eq!(
            names,
            [
                "_tmp", "2fa", "a", "A", "a_b", "a-b", "ab", "b1", "b10", "b2", "city", "City",
                "country", "country_language", "countryflag"
            ]
        );
    }
}
