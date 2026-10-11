//! ClickHouse connector using the HTTP interface. Result sets are read as
//! `JSONEachRow`.

use super::{
    Column, Connector, DEFAULT_CONNECTION_TIMEOUT_MS, Error, QueryResult, Table, prompt_or,
    prompt_timeout, timeout_ms,
};
use crate::guard::base64_encode;
use crate::json;
use crate::prompt;
use crate::url::{Url, encode_uri_component};
use crate::value::{Cell, Row};
use std::time::Duration;

pub const CONNECTOR: Connector = Connector {
    key: "clickhouse",
    database_name: "ClickHouse",
    connection_string_hint,
    prompt_connection_string,
    get_tables,
    query,
};

fn connection_string_hint() -> String {
    format!(
        "http(s)://user:password@host:port?database=database_name
Default port: 8123 (HTTP), 8443 (HTTPS)
Additional url parameters:
'connectionTimeoutMs' - connection timeout in milliseconds (default: {DEFAULT_CONNECTION_TIMEOUT_MS})
More details: https://clickhouse.com/docs/en/interfaces/http"
    )
}

fn prompt_connection_string() -> Result<String, String> {
    let host_input = prompt_or(
        "Database host (default: localhost | example: my.db.com)",
        "localhost",
    )?;
    let lower = host_input.to_ascii_lowercase();
    let protocol_in_host = ["http://", "https://"]
        .into_iter()
        .find(|prefix| lower.starts_with(prefix));
    let host = match protocol_in_host {
        Some(prefix) => &host_input[prefix.len()..],
        None => host_input.as_str(),
    };
    let host = host.trim_end_matches('/').to_string();

    let port = prompt_or("Port (default: 8123)", "8123")?;
    let database = prompt_or("Database name (default: default)", "default")?;
    let user = prompt_or("Username (default: default)", "default")?;
    let password = encode_uri_component(&prompt::secret("Password")?);
    let protocol = match protocol_in_host {
        Some(prefix) => prefix.trim_end_matches("://").to_string(),
        None => prompt::select(
            "Protocol (default: http)",
            &[
                ("http".to_string(), "http".to_string()),
                ("https".to_string(), "https".to_string()),
            ],
        )?,
    };
    let timeout = prompt_timeout()?;
    Ok(format!(
        "{protocol}://{user}:{password}@{host}:{port}?database={database}&connectionTimeoutMs={timeout}"
    ))
}

struct Connection {
    base_url: String,
    database: String,
    authorization: String,
    timeout: Duration,
}

impl Connection {
    fn parse(connection_string: &str) -> Result<Connection, Error> {
        let url = Url::parse(connection_string).map_err(client_error)?;
        let database = url
            .param("database")
            .map(str::to_string)
            .filter(|database| !database.is_empty())
            .or_else(|| Some(url.path_name()).filter(|path| !path.is_empty()))
            .unwrap_or_else(|| "default".to_string());
        let user = Some(url.user())
            .filter(|user| !user.is_empty())
            .unwrap_or_else(|| "default".to_string());
        let credentials = format!("{user}:{}", url.decoded_password());
        Ok(Connection {
            base_url: format!("{}://{}", url.scheme, url.host_with_port()),
            database,
            authorization: format!("Basic {}", base64_encode(credentials.as_bytes())),
            timeout: Duration::from_millis(timeout_ms(url.param("connectionTimeoutMs"))),
        })
    }

    /// Sends a query and returns the response body.
    fn send(&self, sql: &str, params: &[(&str, &str)]) -> Result<String, Error> {
        let mut url = format!(
            "{}/?database={}",
            self.base_url,
            encode_uri_component(&self.database)
        );
        for (key, value) in params {
            url.push_str(&format!("&{key}={}", encode_uri_component(value)));
        }
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(self.timeout))
            .http_status_as_error(false)
            .user_agent(format!("sqlr/{}", crate::VERSION))
            .build()
            .into();
        let mut response = agent
            .post(&url)
            .header("Authorization", &self.authorization)
            .send(sql)
            .map_err(client_error)?;
        let status = response.status();
        let body = response
            .body_mut()
            .with_config()
            .limit(u64::MAX)
            .read_to_string()
            .map_err(client_error)?;
        if !status.is_success() {
            return Err(Error::Database(format!(
                "ClickHouseError: {}",
                truncate_message(&server_message(&body))
            )));
        }
        Ok(body)
    }

    fn select(&self, sql: &str, params: &[(&str, &str)]) -> Result<Vec<Row>, Error> {
        let mut params = params.to_vec();
        params.push(("default_format", "JSONEachRow"));
        let body = self.send(sql, &params)?;
        body.lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| match json::parse(line) {
                Ok(json::Value::Object(object)) => {
                    let mut row = Row::new();
                    for (key, value) in object.iter() {
                        row.insert(key.clone(), Cell::from(value));
                    }
                    Ok(row)
                }
                Ok(_) | Err(_) => Err(Error::Database(format!(
                    "ClickHouseError: Unexpected response: {line}"
                ))),
            })
            .collect()
    }
}

fn client_error(err: impl std::fmt::Display) -> Error {
    Error::Database(format!("ClickHouseError: {err}"))
}

/// The message of a server exception like
/// `Code: 62. DB::Exception: <message> (SYNTAX_ERROR) (version ...)`.
fn server_message(body: &str) -> String {
    let parsed = body
        .find("Code: ")
        .or_else(|| body.find("Error: "))
        .and_then(|start| {
            let after = &body[start..];
            let message_start = start + after.rfind("Exception: ")? + "Exception: ".len();
            let message = &body[message_start..];
            let end = message
                .match_indices('(')
                .filter(|(index, _)| {
                    let rest = &message[index + 1..];
                    let name_len = rest
                        .find(|c: char| !(c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'))
                        .unwrap_or(rest.len());
                    name_len > 0 && rest[name_len..].starts_with(')')
                })
                .map(|(index, _)| index)
                .last()?;
            Some(message[..end].to_string())
        });
    parsed.unwrap_or_else(|| body.trim().to_string())
}

/// Drops the list of expected tokens from syntax errors.
fn truncate_message(message: &str) -> String {
    match message.find("Expected one of:") {
        Some(index) => message[..index].trim().to_string(),
        None => message.trim_end().to_string(),
    }
}

fn is_select_query(sql: &str) -> bool {
    let sql = sql.trim().to_uppercase();
    ["SELECT", "WITH", "SHOW", "DESCRIBE", "EXISTS"]
        .iter()
        .any(|keyword| sql.starts_with(keyword))
}

fn text(row: &Row, key: &str) -> String {
    match row.get(key) {
        Some(Cell::String(value)) => value.clone(),
        Some(other) => crate::value::to_display_string(other),
        None => String::new(),
    }
}

fn get_tables(connection_string: &str) -> Result<Vec<Table>, Error> {
    let connection = Connection::parse(connection_string)?;
    let database = connection.database.clone();
    let params = [("param_database", database.as_str())];
    let tables = connection.select(
        "SELECT database AS table_schema, name AS table_name FROM system.tables WHERE database = {database: String} AND is_temporary = 0 AND name NOT LIKE '.%' ORDER BY database, name",
        &params,
    )?;
    let columns = connection.select(
        "SELECT database AS table_schema, table AS table_name, name AS column_name, default_expression AS column_default, type AS data_type FROM system.columns WHERE database = {database: String} ORDER BY database, table, position",
        &params,
    )?;

    Ok(tables
        .iter()
        .map(|table| {
            let schema = text(table, "table_schema");
            let name = text(table, "table_name");
            let columns = columns
                .iter()
                .filter(|column| {
                    text(column, "table_schema") == schema && text(column, "table_name") == name
                })
                .map(|column| {
                    let data_type = text(column, "data_type");
                    let nullable = data_type
                        .strip_prefix("Nullable(")
                        .and_then(|inner| inner.strip_suffix(')'));
                    Column {
                        name: text(column, "column_name"),
                        data_type: nullable.unwrap_or(&data_type).to_string(),
                        nullable: nullable.is_some(),
                        relation: None,
                    }
                })
                .collect();
            Table {
                schema,
                name,
                columns,
            }
        })
        .collect())
}

fn query(connection_string: &str, sql: &str) -> Result<QueryResult, Error> {
    let connection = Connection::parse(connection_string)?;
    if is_select_query(sql) {
        let rows = connection.select(sql, &[])?;
        return Ok(QueryResult {
            rows_affected: Cell::Number(rows.len() as f64),
            rows,
        });
    }
    connection.send(sql, &[])?;
    Ok(QueryResult {
        rows_affected: Cell::Number(0.0),
        rows: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_connection_strings() {
        let connection =
            Connection::parse("https://user:p%40ss%3Bword@localhost:8443?database=mydb").unwrap();
        assert_eq!(connection.base_url, "https://localhost:8443");
        assert_eq!(connection.database, "mydb");
        assert_eq!(
            connection.authorization,
            format!("Basic {}", base64_encode(b"user:p@ss;word"))
        );
        assert_eq!(connection.timeout, Duration::from_millis(10000));

        let connection = Connection::parse("http://localhost/analytics?connectionTimeoutMs=500").unwrap();
        assert_eq!(connection.database, "analytics");
        assert_eq!(connection.authorization, format!("Basic {}", base64_encode(b"default:")));
        assert_eq!(connection.timeout, Duration::from_millis(500));
    }

    #[test]
    fn extracts_server_messages() {
        let body = "Code: 62. DB::Exception: Syntax error: failed at position 1 (INVALID): INVALID SQL QUERY. Expected one of: Query, SHOW [FULL] TABLES (X). (SYNTAX_ERROR) (version 26.6.1.1193 (official build))\n";
        assert_eq!(
            truncate_message(&server_message(body)),
            "Syntax error: failed at position 1 (INVALID): INVALID SQL QUERY."
        );
        let body = "Code: 60. DB::Exception: Unknown table expression identifier 'nope' in scope SELECT * FROM nope. (UNKNOWN_TABLE) (version 26.6.1.1193 (official build))\n";
        assert_eq!(
            truncate_message(&server_message(body)),
            "Unknown table expression identifier 'nope' in scope SELECT * FROM nope."
        );
        assert_eq!(server_message("Unauthorized\n"), "Unauthorized");
    }

    #[test]
    fn detects_select_queries() {
        assert!(is_select_query("  select 1"));
        assert!(is_select_query("WITH x AS (SELECT 1) SELECT * FROM x"));
        assert!(is_select_query("show tables"));
        assert!(!is_select_query("INSERT INTO t VALUES (1)"));
    }
}
