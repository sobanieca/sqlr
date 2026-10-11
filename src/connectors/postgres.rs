//! PostgreSQL connector. Queries run through the simple query protocol (so
//! scripts with several statements work) and values are converted from their
//! text representation like node-postgres does. Column types come from
//! preparing the statement first.

use super::{
    Column, Connector, DEFAULT_CONNECTION_TIMEOUT_MS, Error, QueryResult, Table, block_on,
    prompt_or, prompt_timeout, timeout_ms, tls_config,
};
use crate::date::DateTime;
use crate::json;
use crate::prompt;
use crate::url::{Url, encode_uri_component};
use crate::value::{Cell, Object, Row};
use std::time::Duration;
use tokio_postgres::config::SslMode;
use tokio_postgres::{Client, Config, SimpleQueryMessage};
use tokio_postgres_rustls::MakeRustlsConnect;

pub const CONNECTOR: Connector = Connector {
    key: "postgresql",
    database_name: "PostgreSQL",
    connection_string_hint,
    prompt_connection_string,
    get_tables,
    query,
};

fn connection_string_hint() -> String {
    format!(
        "postgres://host:port/database_name?user=user&password=password(urlencoded)&application_name=sqlr
Additional url parameters:
'sslmode' - disable | prefer | require | verify-full (default: disable)
'options' - additional values for connection (options=--cluster=your_cluster_name)
'connectionTimeoutMs' - connection timeout in milliseconds (default: {DEFAULT_CONNECTION_TIMEOUT_MS})"
    )
}

fn prompt_connection_string() -> Result<String, String> {
    let host = prompt_or(
        "Database host (default: localhost | example: my.db.com)",
        "localhost",
    )?;
    let port = prompt_or("Port (default: 5432)", "5432")?;
    let database = prompt::input("Database name")?;
    let user = prompt::input("Username")?;
    let password = encode_uri_component(&prompt::secret("Password")?);
    let ssl = if prompt::toggle("Use SSL?")? {
        "&sslmode=prefer"
    } else {
        ""
    };
    let timeout = prompt_timeout()?;
    Ok(format!(
        "postgres://{host}:{port}/{database}?user={user}&password={password}&application_name=sqlr{ssl}&connectionTimeoutMs={timeout}"
    ))
}

enum Tls {
    Disabled,
    /// TLS without certificate verification, falling back to plain text.
    Prefer,
    /// TLS without certificate verification.
    Require,
    /// TLS with certificate verification.
    Verify,
}

fn parse_connection_string(connection_string: &str) -> Result<(Config, Tls), Error> {
    let url = Url::parse(connection_string).map_err(Error::Other)?;
    let mut config = Config::new();
    config.host(if url.host.is_empty() {
        "localhost"
    } else {
        url.hostname()
    });
    config.port(url.port.unwrap_or(5432));
    let database = crate::url::percent_decode(&url.path_name());
    if !database.is_empty() {
        config.dbname(database);
    }
    let user = url
        .param("user")
        .map(str::to_string)
        .unwrap_or_else(|| url.user());
    if !user.is_empty() {
        config.user(user);
    }
    let password = url
        .param("password")
        .map(str::to_string)
        .unwrap_or_else(|| url.decoded_password());
    if !password.is_empty() {
        config.password(password);
    }
    if let Some(name) = url.param("application_name") {
        config.application_name(name);
    }
    if let Some(options) = url.param("options") {
        config.options(options);
    }
    config.connect_timeout(Duration::from_millis(timeout_ms(
        url.param("connectionTimeoutMs"),
    )));
    let tls = match url.param("sslmode").or(url.param("ssl")) {
        None | Some("disable") | Some("false") => Tls::Disabled,
        Some("prefer") | Some("allow") => Tls::Prefer,
        Some("require") | Some("no-verify") => Tls::Require,
        Some(_) => Tls::Verify,
    };
    Ok((config, tls))
}

fn db_error(err: tokio_postgres::Error) -> Error {
    match err.as_db_error() {
        Some(db) => Error::Database(format!("error: {}", db.message())),
        None => Error::Other(err.to_string()),
    }
}

async fn connect(connection_string: &str) -> Result<Client, Error> {
    let (mut config, tls) = parse_connection_string(connection_string)?;
    let connector = match tls {
        Tls::Disabled => {
            config.ssl_mode(SslMode::Disable);
            MakeRustlsConnect::new(tls_config(false))
        }
        Tls::Prefer => {
            config.ssl_mode(SslMode::Prefer);
            MakeRustlsConnect::new(tls_config(false))
        }
        Tls::Require => {
            config.ssl_mode(SslMode::Require);
            MakeRustlsConnect::new(tls_config(false))
        }
        Tls::Verify => {
            config.ssl_mode(SslMode::Require);
            MakeRustlsConnect::new(tls_config(true))
        }
    };
    let (client, connection) = config.connect(connector).await.map_err(db_error)?;
    tokio::spawn(connection);
    Ok(client)
}

/// Runs a query through the simple protocol, returning the column names and
/// rows of each statement with their command counts.
struct Statement {
    columns: Vec<String>,
    rows: Vec<Vec<Option<String>>>,
    count: u64,
}

async fn simple_query(client: &Client, sql: &str) -> Result<Vec<Statement>, Error> {
    let messages = client.simple_query(sql).await.map_err(db_error)?;
    let mut statements = Vec::new();
    let mut current = Statement {
        columns: Vec::new(),
        rows: Vec::new(),
        count: 0,
    };
    for message in messages {
        match message {
            SimpleQueryMessage::RowDescription(columns) => {
                current.columns = columns.iter().map(|c| c.name().to_string()).collect();
            }
            SimpleQueryMessage::Row(row) => {
                current
                    .rows
                    .push((0..row.len()).map(|i| row.get(i).map(str::to_string)).collect());
            }
            SimpleQueryMessage::CommandComplete(count) => {
                current.count = count;
                statements.push(std::mem::replace(
                    &mut current,
                    Statement {
                        columns: Vec::new(),
                        rows: Vec::new(),
                        count: 0,
                    },
                ));
            }
            _ => {}
        }
    }
    Ok(statements)
}

fn is_multiple_statements_error(err: &tokio_postgres::Error) -> bool {
    err.as_db_error()
        .is_some_and(|db| db.message().contains("multiple commands"))
}

/// Commands whose completion tag has a row count (node-postgres reports
/// `null` for the others).
fn has_row_count(sql: &str) -> bool {
    let keyword: String = sql
        .trim_start()
        .chars()
        .take_while(char::is_ascii_alphabetic)
        .collect::<String>()
        .to_uppercase();
    matches!(
        keyword.as_str(),
        "SELECT" | "WITH" | "VALUES" | "TABLE" | "INSERT" | "UPDATE" | "DELETE" | "MERGE"
            | "COPY" | "MOVE" | "FETCH"
    )
}

fn query(connection_string: &str, sql: &str) -> Result<QueryResult, Error> {
    block_on(async {
        let client = connect(connection_string).await?;
        let types = match client.prepare(sql).await {
            Ok(statement) => Some(
                statement
                    .columns()
                    .iter()
                    .map(|column| column.type_().oid())
                    .collect::<Vec<u32>>(),
            ),
            Err(err) if is_multiple_statements_error(&err) => None,
            Err(err) => return Err(db_error(err)),
        };
        let statements = simple_query(&client, sql).await?;

        let Some(types) = types else {
            return Ok(QueryResult {
                rows_affected: Cell::Undefined,
                rows: Vec::new(),
            });
        };
        let Some(statement) = statements.into_iter().next() else {
            return Ok(QueryResult {
                rows_affected: Cell::Null,
                rows: Vec::new(),
            });
        };
        let rows = statement
            .rows
            .iter()
            .map(|values| {
                let mut row = Row::new();
                for (index, name) in statement.columns.iter().enumerate() {
                    let oid = types.get(index).copied().unwrap_or(0);
                    let value = values.get(index).cloned().flatten();
                    row.insert(name.clone(), parse_value(oid, value.as_deref()));
                }
                row
            })
            .collect();
        let rows_affected = if statement.columns.is_empty() && !has_row_count(sql) {
            Cell::Null
        } else {
            Cell::Number(statement.count as f64)
        };
        Ok(QueryResult {
            rows_affected,
            rows,
        })
    })?
}

/// Converts a value in PostgreSQL text format like the default type parsers
/// of node-postgres (`pg-types`).
pub fn parse_value(oid: u32, text: Option<&str>) -> Cell {
    let Some(text) = text else {
        return Cell::Null;
    };
    if let Some(element) = array_element(oid) {
        return parse_array(text, element);
    }
    match oid {
        21 | 23 | 26 => Cell::Number(parse_int(text)),
        700 | 701 => Cell::Number(parse_float(text)),
        16 => Cell::Bool(matches!(
            text,
            "TRUE" | "t" | "true" | "y" | "yes" | "on" | "1"
        )),
        1082 => parse_date(text),
        1114 => parse_timestamp(text, false),
        1184 => parse_timestamp(text, true),
        114 | 3802 => json::parse(text)
            .map(|value| Cell::from(&value))
            .unwrap_or_else(|_| Cell::String(text.to_string())),
        17 => Cell::Buffer(parse_bytea(text)),
        1186 => parse_interval(text),
        600 => parse_point(text),
        _ => Cell::String(text.to_string()),
    }
}

/// Element type of the array types node-postgres parses.
fn array_element(oid: u32) -> Option<u32> {
    Some(match oid {
        1000 => 16,
        1001 => 17,
        1005 => 21,
        1007 => 23,
        1028 => 26,
        1016 => 20,
        1017 => 600,
        1021 => 700,
        1022 => 701,
        1231 => 1700,
        1014 | 1015 | 1009 | 1263 => 25,
        1115 => 1114,
        1182 => 1082,
        1185 => 1184,
        1187 => 1186,
        199 => 114,
        3807 => 3802,
        2951 => 2950,
        791 => 790,
        1041 => 869,
        651 => 650,
        1040 => 829,
        1183 => 1083,
        1270 => 1266,
        _ => return None,
    })
}

/// `parseInt(text, 10)`
fn parse_int(text: &str) -> f64 {
    text.trim().parse::<i64>().map(|n| n as f64).unwrap_or(f64::NAN)
}

/// `parseFloat(text)`
fn parse_float(text: &str) -> f64 {
    match text.trim() {
        "NaN" => f64::NAN,
        "Infinity" => f64::INFINITY,
        "-Infinity" => f64::NEG_INFINITY,
        other => other.parse().unwrap_or(f64::NAN),
    }
}

/// Parses an array literal like `{1,2,NULL}` or `{"a b","c\"d"}`.
fn parse_array(text: &str, element: u32) -> Cell {
    let mut chars = text.chars().peekable();
    if text.starts_with('[') {
        while let Some(c) = chars.next() {
            if c == '=' {
                break;
            }
        }
    }
    parse_array_level(&mut chars, element).unwrap_or_else(|| Cell::String(text.to_string()))
}

fn parse_array_level(chars: &mut std::iter::Peekable<std::str::Chars>, element: u32) -> Option<Cell> {
    if chars.next()? != '{' {
        return None;
    }
    let mut items = Vec::new();
    if chars.peek() == Some(&'}') {
        chars.next();
        return Some(Cell::Array(items));
    }
    loop {
        match chars.peek()? {
            '{' => items.push(parse_array_level(chars, element)?),
            '"' => {
                chars.next();
                let mut value = String::new();
                loop {
                    match chars.next()? {
                        '\\' => value.push(chars.next()?),
                        '"' => break,
                        c => value.push(c),
                    }
                }
                items.push(parse_value(element, Some(&value)));
            }
            _ => {
                let mut value = String::new();
                while let Some(&c) = chars.peek() {
                    if c == ',' || c == '}' {
                        break;
                    }
                    value.push(c);
                    chars.next();
                }
                if value == "NULL" {
                    items.push(Cell::Null);
                } else {
                    items.push(parse_value(element, Some(&value)));
                }
            }
        }
        match chars.next()? {
            ',' => continue,
            '}' => return Some(Cell::Array(items)),
            _ => return None,
        }
    }
}

/// `YYYY-MM-DD` as local midnight. Infinite dates become numbers.
fn parse_date(text: &str) -> Cell {
    match text {
        "infinity" => return Cell::Number(f64::INFINITY),
        "-infinity" => return Cell::Number(f64::NEG_INFINITY),
        _ => {}
    }
    let (text, bc) = match text.strip_suffix(" BC") {
        Some(text) => (text, true),
        None => (text, false),
    };
    match parse_ymd(text) {
        Some((year, month, day)) => {
            let year = if bc { 1 - year } else { year };
            Cell::Date(DateTime::date(year, month, day).to_local_ms())
        }
        None => Cell::String(text.to_string()),
    }
}

fn parse_ymd(text: &str) -> Option<(i64, i64, i64)> {
    let mut parts = text.splitn(3, '-');
    let year = parts.next()?.parse().ok()?;
    let month = parts.next()?.parse().ok()?;
    let day = parts.next()?.get(..2)?.parse().ok()?;
    Some((year, month, day))
}

/// `YYYY-MM-DD HH:MM:SS[.ffffff][+HH[:MM[:SS]]][ BC]`: with an offset the
/// time is absolute, otherwise local time.
fn parse_timestamp(text: &str, with_time_zone: bool) -> Cell {
    match text {
        "infinity" => return Cell::Number(f64::INFINITY),
        "-infinity" => return Cell::Number(f64::NEG_INFINITY),
        _ => {}
    }
    let parse = || -> Option<f64> {
        let (text, bc) = match text.strip_suffix(" BC") {
            Some(text) => (text, true),
            None => (text, false),
        };
        let (date, time) = text.split_once(' ')?;
        let (year, month, day) = parse_ymd(date)?;
        let year = if bc { 1 - year } else { year };
        let offset_start = time.find(['+', '-', 'Z']).unwrap_or(time.len());
        let (clock, offset) = time.split_at(offset_start);
        let mut clock_parts = clock.splitn(3, ':');
        let hour = clock_parts.next()?.parse().ok()?;
        let minute = clock_parts.next()?.parse().ok()?;
        let seconds = clock_parts.next()?;
        let (second, fraction) = seconds.split_once('.').unwrap_or((seconds, ""));
        let millisecond = format!("{fraction:0<3}")[..3].parse().ok()?;
        let dt = DateTime {
            year,
            month,
            day,
            hour,
            minute,
            second: second.parse().ok()?,
            millisecond,
        };
        if offset.is_empty() {
            return Some(if with_time_zone {
                dt.to_utc_ms()
            } else {
                dt.to_local_ms()
            });
        }
        if offset == "Z" {
            return Some(dt.to_utc_ms());
        }
        let sign = if offset.starts_with('-') { -1 } else { 1 };
        let mut offset_parts = offset[1..].split(':');
        let hours: i64 = offset_parts.next()?.parse().ok()?;
        let minutes: i64 = offset_parts.next().map_or(Some(0), |m| m.parse().ok())?;
        let seconds: i64 = offset_parts.next().map_or(Some(0), |s| s.parse().ok())?;
        let offset_ms = sign * (hours * 3_600_000 + minutes * 60_000 + seconds * 1000);
        Some(dt.to_utc_ms() - offset_ms as f64)
    };
    match parse() {
        Some(ms) => Cell::Date(ms),
        None => Cell::String(text.to_string()),
    }
}

/// Hex (`\x0102`) or escape format.
fn parse_bytea(text: &str) -> Vec<u8> {
    if let Some(hex) = text.strip_prefix("\\x") {
        return (0..hex.len() / 2)
            .filter_map(|i| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok())
            .collect();
    }
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            if bytes.get(i + 1) == Some(&b'\\') {
                out.push(b'\\');
                i += 2;
                continue;
            }
            if let Some(octal) = text.get(i + 1..i + 4)
                && let Ok(byte) = u8::from_str_radix(octal, 8)
            {
                out.push(byte);
                i += 4;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    out
}

/// `postgres-interval`: an object with the non-zero parts of the interval.
fn parse_interval(text: &str) -> Cell {
    let mut years = 0.0;
    let mut months = 0.0;
    let mut days = 0.0;
    let mut hours = 0.0;
    let mut minutes = 0.0;
    let mut seconds = 0.0;
    let mut milliseconds = 0.0;
    let tokens: Vec<&str> = text.split_whitespace().collect();
    let mut i = 0;
    while i < tokens.len() {
        let token = tokens[i];
        if token.contains(':') {
            let negative = token.starts_with('-');
            let parts: Vec<&str> = token.trim_start_matches(['-', '+']).split(':').collect();
            let sign = if negative { -1.0 } else { 1.0 };
            hours = sign * parts.first().and_then(|h| h.parse::<f64>().ok()).unwrap_or(0.0);
            minutes = sign * parts.get(1).and_then(|m| m.parse::<f64>().ok()).unwrap_or(0.0);
            if let Some(s) = parts.get(2) {
                let (whole, fraction) = s.split_once('.').unwrap_or((s, ""));
                seconds = sign * whole.parse::<f64>().unwrap_or(0.0);
                if !fraction.is_empty() {
                    let fraction: f64 = format!("0.{fraction}").parse().unwrap_or(0.0);
                    milliseconds = sign * (fraction * 1000.0 * 1000.0).round() / 1000.0;
                }
            }
            i += 1;
            continue;
        }
        let value: f64 = token.parse().unwrap_or(0.0);
        let unit = tokens.get(i + 1).copied().unwrap_or_default();
        match unit.trim_end_matches('s') {
            "year" => years = value,
            "mon" => months = value,
            "day" => days = value,
            _ => {}
        }
        i += 2;
    }
    let mut object = Object::with_class("PostgresInterval");
    for (name, value) in [
        ("years", years),
        ("months", months),
        ("days", days),
        ("hours", hours),
        ("minutes", minutes),
        ("seconds", seconds),
        ("milliseconds", milliseconds),
    ] {
        if value != 0.0 {
            object.insert(name, Cell::Number(value));
        }
    }
    Cell::Object(object)
}

/// `(x,y)` as `{ x, y }`.
fn parse_point(text: &str) -> Cell {
    let inner = text.trim_matches(['(', ')']);
    match inner.split_once(',') {
        Some((x, y)) => {
            let mut object = Object::new();
            object.insert("x", Cell::Number(parse_float(x)));
            object.insert("y", Cell::Number(parse_float(y)));
            Cell::Object(object)
        }
        None => Cell::String(text.to_string()),
    }
}

fn get_tables(connection_string: &str) -> Result<Vec<Table>, Error> {
    block_on(async {
        let client = connect(connection_string).await?;
        let run = |sql: &'static str| {
            let client = &client;
            async move {
                let statement = simple_query(client, sql).await?.into_iter().next();
                Ok::<_, Error>(statement.map_or_else(Vec::new, |statement| {
                    statement
                        .rows
                        .into_iter()
                        .map(|row| {
                            statement
                                .columns
                                .iter()
                                .cloned()
                                .zip(row.into_iter().map(Option::unwrap_or_default))
                                .collect::<Vec<(String, String)>>()
                        })
                        .collect::<Vec<_>>()
                }))
            }
        };
        let tables = run(TABLES_QUERY).await?;
        let columns = run(COLUMNS_QUERY).await?;
        let foreign_keys = run(FOREIGN_KEYS_QUERY).await?;

        let get = |row: &Vec<(String, String)>, key: &str| -> String {
            row.iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
                .unwrap_or_default()
        };
        Ok(tables
            .iter()
            .map(|table| {
                let schema = get(table, "table_schema");
                let name = get(table, "table_name");
                let columns = columns
                    .iter()
                    .filter(|column| {
                        get(column, "table_schema") == schema && get(column, "table_name") == name
                    })
                    .map(|column| {
                        let column_name = get(column, "column_name");
                        let relation = foreign_keys
                            .iter()
                            .find(|fk| {
                                get(fk, "table_schema") == schema
                                    && get(fk, "table_name") == name
                                    && get(fk, "column_name") == column_name
                            })
                            .map(|fk| {
                                format!(
                                    "{}.{}.{}",
                                    get(fk, "foreign_table_schema"),
                                    get(fk, "foreign_table_name"),
                                    get(fk, "foreign_column_name")
                                )
                            });
                        Column {
                            name: column_name,
                            data_type: get(column, "data_type"),
                            nullable: get(column, "is_nullable") == "YES",
                            relation,
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
    })?
}

const TABLES_QUERY: &str = "select table_schema, table_name from information_schema.tables where table_schema not in ('pg_catalog', 'information_schema') order by table_schema, table_name";

const COLUMNS_QUERY: &str = "select table_schema, table_name, column_name, column_default, is_nullable, data_type from information_schema.columns where table_schema not in ('pg_catalog', 'information_schema') order by table_schema, table_name, ordinal_position";

const FOREIGN_KEYS_QUERY: &str = "select tc.table_schema, tc.constraint_name, tc.table_name, kcu.column_name, ccu.table_schema AS foreign_table_schema, ccu.table_name AS foreign_table_name, ccu.column_name AS foreign_column_name from information_schema.table_constraints as tc join information_schema.key_column_usage as kcu on tc.constraint_name = kcu.constraint_name and tc.table_schema = kcu.table_schema join information_schema.constraint_column_usage as ccu on ccu.constraint_name = tc.constraint_name and ccu.table_schema = tc.table_schema where tc.constraint_type = 'FOREIGN KEY'";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::to_iso_string;

    #[test]
    fn parses_scalars() {
        assert_eq!(parse_value(23, Some("2")), Cell::Number(2.0));
        assert_eq!(parse_value(20, Some("9007199254740993")), Cell::String("9007199254740993".into()));
        assert_eq!(parse_value(1700, Some("12.345")), Cell::String("12.345".into()));
        assert_eq!(parse_value(16, Some("t")), Cell::Bool(true));
        assert_eq!(parse_value(16, Some("f")), Cell::Bool(false));
        assert!(matches!(parse_value(701, Some("NaN")), Cell::Number(n) if n.is_nan()));
        assert_eq!(parse_value(701, Some("-Infinity")), Cell::Number(f64::NEG_INFINITY));
        assert_eq!(parse_value(25, None), Cell::Null);
        assert_eq!(parse_value(17, Some("\\x0102ff")), Cell::Buffer(vec![1, 2, 255]));
        assert_eq!(parse_bytea("a\\\\b\\001"), b"a\\b\x01".to_vec());
    }

    #[test]
    fn parses_timestamps() {
        let Cell::Date(ms) = parse_value(1184, Some("2024-01-02 01:04:05.678+00")) else {
            panic!()
        };
        assert_eq!(to_iso_string(ms).unwrap(), "2024-01-02T01:04:05.678Z");
        let Cell::Date(ms) = parse_value(1184, Some("2024-01-02 03:04:05.1234+05:30")) else {
            panic!()
        };
        assert_eq!(to_iso_string(ms).unwrap(), "2024-01-01T21:34:05.123Z");
        assert_eq!(parse_value(1114, Some("infinity")), Cell::Number(f64::INFINITY));
    }

    #[test]
    fn parses_arrays() {
        assert_eq!(
            parse_value(1007, Some("{1,NULL,3}")),
            Cell::Array(vec![Cell::Number(1.0), Cell::Null, Cell::Number(3.0)])
        );
        assert_eq!(
            parse_value(1009, Some(r#"{"a b","c\"d",NULL,"NULL"}"#)),
            Cell::Array(vec![
                Cell::String("a b".into()),
                Cell::String("c\"d".into()),
                Cell::Null,
                Cell::String("NULL".into())
            ])
        );
        assert_eq!(
            parse_value(1007, Some("{{1,2},{3,4}}")),
            Cell::Array(vec![
                Cell::Array(vec![Cell::Number(1.0), Cell::Number(2.0)]),
                Cell::Array(vec![Cell::Number(3.0), Cell::Number(4.0)])
            ])
        );
        assert_eq!(parse_value(1007, Some("{}")), Cell::Array(vec![]));
    }

    #[test]
    fn parses_intervals() {
        let Cell::Object(interval) = parse_value(1186, Some("1 day 02:03:04")) else {
            panic!()
        };
        assert_eq!(interval.class.as_deref(), Some("PostgresInterval"));
        let keys: Vec<&String> = interval.keys().collect();
        assert_eq!(keys, ["days", "hours", "minutes", "seconds"]);
        let Cell::Object(interval) = parse_value(1186, Some("1 year 2 mons -00:00:01.5")) else {
            panic!()
        };
        assert_eq!(interval.get("years"), Some(&Cell::Number(1.0)));
        assert_eq!(interval.get("months"), Some(&Cell::Number(2.0)));
        assert_eq!(interval.get("seconds"), Some(&Cell::Number(-1.0)));
        assert_eq!(interval.get("milliseconds"), Some(&Cell::Number(-500.0)));
    }

    #[test]
    fn parses_connection_strings() {
        let (config, _) = parse_connection_string(
            "postgres://localhost:5432/db?user=admin&password=p%40ss%3Aw%23rd&application_name=sqlr&sslmode=prefer",
        )
        .unwrap();
        assert_eq!(config.get_user(), Some("admin"));
        assert_eq!(config.get_password(), Some(&b"p@ss:w#rd"[..]));
        assert_eq!(config.get_dbname(), Some("db"));
        assert_eq!(config.get_ports(), [5432]);

        let (config, _) = parse_connection_string("postgres://u:p%40@db.host/world-db").unwrap();
        assert_eq!(config.get_user(), Some("u"));
        assert_eq!(config.get_password(), Some(&b"p@"[..]));
        assert!(parse_connection_string("not a url").is_err());
    }
}
