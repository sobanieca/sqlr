//! MySQL connector. Queries use the text protocol and values are converted
//! like mysql2 does with `supportBigNumbers` and `bigNumberStrings`.

use super::{
    Column, Connector, DEFAULT_CONNECTION_TIMEOUT_MS, Error, QueryResult, Table, block_on,
    prompt_or, prompt_timeout, timeout_ms,
};
use crate::date::DateTime;
use crate::json;
use crate::prompt;
use crate::url::{Url, encode_uri_component, percent_decode};
use crate::value::{Cell, Row};
use mysql_async::consts::ColumnType;
use mysql_async::prelude::Queryable;
use mysql_async::{Conn, OptsBuilder, SslOpts};
use std::time::Duration;

pub const CONNECTOR: Connector = Connector {
    key: "mysql",
    database_name: "MySQL",
    connection_string_hint,
    prompt_connection_string,
    get_tables,
    query,
};

/// Character set of binary strings and blobs.
const BINARY_CHARSET: u16 = 63;

fn connection_string_hint() -> String {
    format!(
        "mysql://user:password@host:port/database_name
Additional url parameters:
'ssl' - {{\"rejectUnauthorized\":true|false}}
'connectionTimeoutMs' - connection timeout in milliseconds (default: {DEFAULT_CONNECTION_TIMEOUT_MS})"
    )
}

fn prompt_connection_string() -> Result<String, String> {
    let host = prompt_or(
        "Database host (default: localhost | example: my.db.com)",
        "localhost",
    )?;
    let port = prompt_or("Port (default: 3306)", "3306")?;
    let database = prompt::input("Database name")?;
    let user = prompt::input("Username")?;
    let password = encode_uri_component(&prompt::secret("Password")?);
    let ssl = prompt::select(
        "SSL mode (default: disabled)",
        &[
            ("disabled".to_string(), "disabled".to_string()),
            (
                "enabled (verify certificate)".to_string(),
                "verify".to_string(),
            ),
            (
                "enabled (trust any certificate)".to_string(),
                "trust".to_string(),
            ),
        ],
    )?;
    let timeout = prompt_timeout()?;
    let ssl = match ssl.as_str() {
        "verify" => "ssl={\"rejectUnauthorized\":true}&",
        "trust" => "ssl={\"rejectUnauthorized\":false}&",
        _ => "",
    };
    Ok(format!(
        "mysql://{user}:{password}@{host}:{port}/{database}?{ssl}connectionTimeoutMs={timeout}"
    ))
}

struct Connection {
    opts: OptsBuilder,
    database: String,
    timeout: Duration,
}

fn parse_connection_string(connection_string: &str) -> Result<Connection, Error> {
    let url = Url::parse(connection_string).map_err(Error::Other)?;
    let database = percent_decode(&url.path_name());
    let mut opts = OptsBuilder::default()
        .ip_or_hostname(if url.host.is_empty() {
            "localhost"
        } else {
            url.hostname()
        })
        .tcp_port(url.port.unwrap_or(3306))
        .user(Some(url.user()).filter(|user| !user.is_empty()))
        .pass(Some(url.decoded_password()).filter(|pass| !pass.is_empty()))
        .db_name(Some(database.clone()).filter(|db| !db.is_empty()))
        .prefer_socket(false)
        .client_found_rows(true);
    if let Some(ssl) = url.param("ssl") {
        let reject_unauthorized = match json::parse(ssl) {
            Ok(json::Value::Object(options)) => options.get("rejectUnauthorized") != Some(&json::Value::Bool(false)),
            _ => true,
        };
        let ssl_opts = SslOpts::default()
            .with_danger_accept_invalid_certs(!reject_unauthorized)
            .with_danger_skip_domain_validation(!reject_unauthorized);
        opts = opts.ssl_opts(Some(ssl_opts));
    }
    Ok(Connection {
        opts,
        database,
        timeout: Duration::from_millis(timeout_ms(url.param("connectionTimeoutMs"))),
    })
}

fn mysql_error(err: mysql_async::Error) -> Error {
    match err {
        mysql_async::Error::Server(err) => Error::Database(format!("MySQL Error: {}", err.message)),
        other => Error::Other(other.to_string()),
    }
}

async fn connect(connection: &Connection) -> Result<Conn, Error> {
    match tokio::time::timeout(connection.timeout, Conn::new(connection.opts.clone())).await {
        Ok(result) => result.map_err(mysql_error),
        Err(_) => Err(Error::Other("connect ETIMEDOUT".to_string())),
    }
}

/// Runs a query and returns the first result set (column names, types and
/// raw values) or the number of affected rows.
async fn run(conn: &mut Conn, sql: &str) -> Result<RawResult, Error> {
    let mut result = conn.query_iter(sql).await.map_err(mysql_error)?;
    let columns: Vec<(String, ColumnType, u16)> = result
        .columns_ref()
        .iter()
        .map(|column| {
            (
                column.name_str().into_owned(),
                column.column_type(),
                column.character_set(),
            )
        })
        .collect();
    let has_columns = !columns.is_empty();
    let mut rows = Vec::new();
    while let Some(row) = result.next().await.map_err(mysql_error)? {
        rows.push(
            (0..row.len())
                .map(|index| match row.as_ref(index) {
                    Some(mysql_async::Value::Bytes(bytes)) => Some(bytes.clone()),
                    Some(mysql_async::Value::NULL) | None => None,
                    Some(other) => Some(other.as_sql(true).into_bytes()),
                })
                .collect(),
        );
    }
    let affected_rows = result.affected_rows();
    result.drop_result().await.map_err(mysql_error)?;
    Ok(RawResult {
        columns,
        rows,
        has_columns,
        affected_rows,
    })
}

struct RawResult {
    columns: Vec<(String, ColumnType, u16)>,
    rows: Vec<Vec<Option<Vec<u8>>>>,
    has_columns: bool,
    affected_rows: u64,
}

fn query(connection_string: &str, sql: &str) -> Result<QueryResult, Error> {
    let connection = parse_connection_string(connection_string)?;
    block_on(async {
        let mut conn = connect(&connection).await?;
        let result = run(&mut conn, sql).await;
        let _ = conn.disconnect().await;
        let result = result?;
        if !result.has_columns {
            return Ok(QueryResult {
                rows_affected: Cell::Number(result.affected_rows as f64),
                rows: Vec::new(),
            });
        }
        let rows: Vec<Row> = result
            .rows
            .iter()
            .map(|values| {
                let mut row = Row::new();
                for ((name, column_type, charset), value) in result.columns.iter().zip(values) {
                    row.insert(
                        name.clone(),
                        parse_value(*column_type, *charset, value.as_deref()),
                    );
                }
                row
            })
            .collect();
        Ok(QueryResult {
            rows_affected: Cell::Number(rows.len() as f64),
            rows,
        })
    })?
}

/// Converts a text protocol value like mysql2's text parser.
fn parse_value(column_type: ColumnType, charset: u16, value: Option<&[u8]>) -> Cell {
    let Some(bytes) = value else {
        return Cell::Null;
    };
    let text = || String::from_utf8_lossy(bytes).into_owned();
    match column_type {
        ColumnType::MYSQL_TYPE_TINY
        | ColumnType::MYSQL_TYPE_SHORT
        | ColumnType::MYSQL_TYPE_INT24
        | ColumnType::MYSQL_TYPE_LONG
        | ColumnType::MYSQL_TYPE_YEAR => Cell::Number(text().parse::<f64>().unwrap_or(f64::NAN)),
        ColumnType::MYSQL_TYPE_FLOAT | ColumnType::MYSQL_TYPE_DOUBLE => {
            Cell::Number(text().parse::<f64>().unwrap_or(f64::NAN))
        }
        ColumnType::MYSQL_TYPE_LONGLONG
        | ColumnType::MYSQL_TYPE_DECIMAL
        | ColumnType::MYSQL_TYPE_NEWDECIMAL => Cell::String(text()),
        ColumnType::MYSQL_TYPE_DATE | ColumnType::MYSQL_TYPE_NEWDATE => {
            Cell::Date(parse_date_time(&text()))
        }
        ColumnType::MYSQL_TYPE_DATETIME
        | ColumnType::MYSQL_TYPE_DATETIME2
        | ColumnType::MYSQL_TYPE_TIMESTAMP
        | ColumnType::MYSQL_TYPE_TIMESTAMP2 => Cell::Date(parse_date_time(&text())),
        ColumnType::MYSQL_TYPE_JSON => json::parse(&text())
            .map(|value| Cell::from(&value))
            .unwrap_or_else(|_| Cell::String(text())),
        ColumnType::MYSQL_TYPE_BIT | ColumnType::MYSQL_TYPE_GEOMETRY => {
            Cell::Buffer(bytes.to_vec())
        }
        ColumnType::MYSQL_TYPE_TINY_BLOB
        | ColumnType::MYSQL_TYPE_MEDIUM_BLOB
        | ColumnType::MYSQL_TYPE_LONG_BLOB
        | ColumnType::MYSQL_TYPE_BLOB
        | ColumnType::MYSQL_TYPE_VAR_STRING
        | ColumnType::MYSQL_TYPE_VARCHAR
        | ColumnType::MYSQL_TYPE_STRING
            if charset == BINARY_CHARSET =>
        {
            Cell::Buffer(bytes.to_vec())
        }
        _ => Cell::String(text()),
    }
}

/// `YYYY-MM-DD[ HH:MM:SS[.ffffff]]` in local time; zero dates are invalid.
fn parse_date_time(text: &str) -> f64 {
    let parse = || -> Option<DateTime> {
        let (date, time) = text.split_once(' ').unwrap_or((text, "00:00:00"));
        let mut date_parts = date.splitn(3, '-');
        let year = date_parts.next()?.parse().ok()?;
        let month = date_parts.next()?.parse().ok()?;
        let day = date_parts.next()?.parse().ok()?;
        let mut time_parts = time.splitn(3, ':');
        let hour = time_parts.next()?.parse().ok()?;
        let minute = time_parts.next()?.parse().ok()?;
        let seconds = time_parts.next()?;
        let (second, fraction) = seconds.split_once('.').unwrap_or((seconds, ""));
        Some(DateTime {
            year,
            month,
            day,
            hour,
            minute,
            second: second.parse().ok()?,
            millisecond: format!("{fraction:0<3}")[..3].parse().ok()?,
        })
    };
    match parse() {
        Some(dt) if dt.month > 0 && dt.day > 0 => dt.to_local_ms(),
        _ => f64::NAN,
    }
}

fn get_tables(connection_string: &str) -> Result<Vec<Table>, Error> {
    let connection = parse_connection_string(connection_string)?;
    let database = connection.database.replace('\\', "\\\\").replace('\'', "\\'");
    block_on(async {
        let mut conn = connect(&connection).await?;
        let mut fetch = async |sql: String| -> Result<Vec<Vec<(String, String)>>, Error> {
            let result = run(&mut conn, &sql).await?;
            Ok(result
                .rows
                .iter()
                .map(|row| {
                    result
                        .columns
                        .iter()
                        .zip(row)
                        .map(|((name, _, _), value)| {
                            (
                                name.to_lowercase(),
                                value
                                    .as_deref()
                                    .map(|v| String::from_utf8_lossy(v).into_owned())
                                    .unwrap_or_default(),
                            )
                        })
                        .collect()
                })
                .collect())
        };
        let tables = fetch(format!("SELECT table_schema, table_name FROM information_schema.tables WHERE table_schema = '{database}' AND table_type = 'BASE TABLE' ORDER BY table_schema, table_name")).await?;
        let columns = fetch(format!("SELECT table_schema, table_name, column_name, column_default, is_nullable, data_type FROM information_schema.columns WHERE table_schema = '{database}' ORDER BY table_schema, table_name, ordinal_position")).await?;
        let foreign_keys = fetch(format!("SELECT table_schema, table_name, column_name, referenced_table_schema AS foreign_table_schema, referenced_table_name AS foreign_table_name, referenced_column_name AS foreign_column_name FROM information_schema.key_column_usage WHERE table_schema = '{database}' AND referenced_table_name IS NOT NULL")).await?;
        let _ = conn.disconnect().await;

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
                    .filter(|c| get(c, "table_schema") == schema && get(c, "table_name") == name)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::to_iso_string;

    #[test]
    fn converts_values_like_mysql2() {
        let utf8 = 255;
        assert_eq!(
            parse_value(ColumnType::MYSQL_TYPE_LONGLONG, utf8, Some(b"1")),
            Cell::String("1".into())
        );
        assert_eq!(
            parse_value(ColumnType::MYSQL_TYPE_LONG, utf8, Some(b"103000")),
            Cell::Number(103000.0)
        );
        assert_eq!(
            parse_value(ColumnType::MYSQL_TYPE_NEWDECIMAL, utf8, Some(b"828.00")),
            Cell::String("828.00".into())
        );
        assert_eq!(
            parse_value(ColumnType::MYSQL_TYPE_VAR_STRING, BINARY_CHARSET, Some(&[1, 2, 255])),
            Cell::Buffer(vec![1, 2, 255])
        );
        assert_eq!(
            parse_value(ColumnType::MYSQL_TYPE_VAR_STRING, utf8, Some(b"txt")),
            Cell::String("txt".into())
        );
        assert_eq!(parse_value(ColumnType::MYSQL_TYPE_LONG, utf8, None), Cell::Null);
        assert!(matches!(
            parse_value(ColumnType::MYSQL_TYPE_DATE, utf8, Some(b"0000-00-00")),
            Cell::Date(ms) if ms.is_nan()
        ));
    }

    #[test]
    fn parses_dates_in_local_time() {
        if std::env::var("TZ").as_deref() != Ok("UTC") {
            return;
        }
        assert_eq!(
            to_iso_string(parse_date_time("2024-01-02 03:04:05.678")).unwrap(),
            "2024-01-02T03:04:05.678Z"
        );
    }

    #[test]
    fn parses_connection_strings() {
        let connection = parse_connection_string(
            r#"mysql://u:p%40ss@localhost:3306/db?ssl={"rejectUnauthorized":false}&connectionTimeoutMs=5000"#,
        )
        .unwrap();
        assert_eq!(connection.database, "db");
        assert_eq!(connection.timeout, Duration::from_millis(5000));
        let opts = mysql_async::Opts::from(connection.opts);
        assert_eq!(opts.user(), Some("u"));
        assert_eq!(opts.pass(), Some("p@ss"));
        assert!(opts.ssl_opts().is_some_and(|ssl| ssl.accept_invalid_certs()));
    }
}
