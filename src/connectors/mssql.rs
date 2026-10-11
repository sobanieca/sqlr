//! Microsoft SQL Server connector. Queries run through `sp_executesql` so
//! all result sets and row counts are available; values are converted like
//! the mssql/tedious driver does (dates in UTC, bigint as string).

use super::{
    Column, Connector, DEFAULT_CONNECTION_TIMEOUT_MS, Error, QueryResult, Table, block_on,
    prompt_or, prompt_timeout, timeout_ms,
};
use crate::date::days_from_civil;
use crate::prompt;
use crate::url::{Url, encode_uri_component};
use crate::value::{Cell, Row};
use std::time::Duration;
use tiberius::{AuthMethod, Client, ColumnData, Config, EncryptionLevel};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

pub const CONNECTOR: Connector = Connector {
    key: "mssql",
    database_name: "MS SQL Server",
    connection_string_hint,
    prompt_connection_string,
    get_tables,
    query,
};

fn connection_string_hint() -> String {
    format!(
        "mssql://user:password@host:port/database_name
Additional url parameters:
'encrypt' - true | false (default: true)
'trustServerCertificate' - true | false (default: false)
'connectionTimeoutMs' - connection timeout in milliseconds (default: {DEFAULT_CONNECTION_TIMEOUT_MS})"
    )
}

fn boolean_options(default: &str) -> Vec<(String, String)> {
    let other = if default == "true" { "false" } else { "true" };
    [default, other]
        .iter()
        .map(|value| (value.to_string(), value.to_string()))
        .collect()
}

fn prompt_connection_string() -> Result<String, String> {
    let host = prompt_or(
        "Database host (default: localhost | example: my.db.com)",
        "localhost",
    )?;
    let port = prompt_or("Port (default: 1433)", "1433")?;
    let database = prompt::input("Database name")?;
    let user = prompt::input("Username")?;
    let password = encode_uri_component(&prompt::secret("Password")?);
    let encrypt = prompt::select(
        "Encrypt connection (default: true)",
        &boolean_options("true"),
    )?;
    let trust = prompt::select(
        "Trust server certificate (default: false)",
        &boolean_options("false"),
    )?;
    let timeout = prompt_timeout()?;
    Ok(format!(
        "mssql://{user}:{password}@{host}:{port}/{database}?encrypt={encrypt}&trustServerCertificate={trust}&connectionTimeoutMs={timeout}"
    ))
}

fn parse_connection_string(connection_string: &str) -> Result<(Config, Duration), Error> {
    let url = Url::parse(connection_string).map_err(Error::Other)?;
    let mut config = Config::new();
    config.host(url.hostname());
    config.port(url.port.unwrap_or(1433));
    let database = url.path_name();
    if !database.is_empty() {
        config.database(database);
    }
    config.authentication(AuthMethod::sql_server(url.user(), url.decoded_password()));
    config.encryption(if url.param("encrypt") == Some("false") {
        EncryptionLevel::Off
    } else {
        EncryptionLevel::Required
    });
    if url.param("trustServerCertificate") == Some("true") {
        config.trust_cert();
    }
    config.application_name("sqlr");
    let timeout = Duration::from_millis(timeout_ms(url.param("connectionTimeoutMs")));
    Ok((config, timeout))
}

fn server_error(err: tiberius::error::Error) -> Error {
    match err {
        tiberius::error::Error::Server(token) => {
            Error::Database(format!("MSSQLError: {}", token.message()))
        }
        other => Error::Other(other.to_string()),
    }
}

async fn connect(connection_string: &str) -> Result<(Client<Compat<TcpStream>>, Duration), Error> {
    let (config, timeout) = parse_connection_string(connection_string)?;
    let connect = async {
        let tcp = TcpStream::connect(config.get_addr())
            .await
            .map_err(Error::other)?;
        tcp.set_nodelay(true).map_err(Error::other)?;
        Client::connect(config, tcp.compat_write())
            .await
            .map_err(Error::other)
    };
    match tokio::time::timeout(timeout, connect).await {
        Ok(client) => Ok((client?, timeout)),
        Err(_) => Err(Error::Other(format!(
            "Failed to connect in {}ms",
            timeout.as_millis()
        ))),
    }
}

/// Runs a batch and returns its result sets and row counts.
async fn execute(
    client: &mut Client<Compat<TcpStream>>,
    timeout: Duration,
    sql: &str,
) -> Result<(Vec<Vec<Row>>, Vec<u64>), Error> {
    let run = async {
        let mut command = tiberius::Command::new("sp_executesql");
        command.bind_param("@stmt", sql.to_string());
        let result = command
            .exec(client)
            .await
            .map_err(server_error)?
            .into_command_result()
            .await
            .map_err(server_error)?;
        let mut sets = Vec::new();
        let mut index = 0;
        while let Some(rows) = result.to_query_result(index) {
            sets.push(rows.iter().map(row_object).collect());
            index += 1;
        }
        Ok((sets, result.rows_affected().to_vec()))
    };
    tokio::time::timeout(timeout, run)
        .await
        .unwrap_or_else(|_| {
            Err(Error::Other(format!(
                "Timeout: Request failed to complete in {}ms",
                timeout.as_millis()
            )))
        })
}

fn query(connection_string: &str, sql: &str) -> Result<QueryResult, Error> {
    block_on(async {
        let (mut client, timeout) = connect(connection_string).await?;
        let (sets, rows_affected) = execute(&mut client, timeout, sql).await?;
        let _ = client.close().await;
        let rows = sets.into_iter().next().unwrap_or_default();
        Ok(QueryResult {
            rows_affected: Cell::Number(rows_affected.iter().sum::<u64>() as f64),
            rows,
        })
    })?
}

fn row_object(row: &tiberius::Row) -> Row {
    let mut object = Row::new();
    for (column, value) in row.cells() {
        object.insert(column.name(), cell(value.clone()));
    }
    object
}

/// Days between 0001-01-01 and 1970-01-01.
fn days_since_year_one() -> i64 {
    -days_from_civil(1, 1, 1)
}

const MS_PER_DAY: f64 = 86_400_000.0;

fn time_ms(time: tiberius::time::Time) -> f64 {
    let scale = 10u64.pow(u32::from(time.scale()));
    (time.increments() as f64 * 1000.0 / scale as f64).floor()
}

fn date_ms(date: tiberius::time::Date) -> f64 {
    (i64::from(date.days()) - days_since_year_one()) as f64 * MS_PER_DAY
}

fn cell(value: ColumnData<'static>) -> Cell {
    let days_1900 = days_from_civil(1900, 1, 1) as f64;
    match value {
        ColumnData::U8(v) => v.map_or(Cell::Null, |v| Cell::Number(f64::from(v))),
        ColumnData::I16(v) => v.map_or(Cell::Null, |v| Cell::Number(f64::from(v))),
        ColumnData::I32(v) => v.map_or(Cell::Null, |v| Cell::Number(f64::from(v))),
        ColumnData::I64(v) => v.map_or(Cell::Null, |v| Cell::String(v.to_string())),
        ColumnData::F32(v) => v.map_or(Cell::Null, |v| Cell::Number(f64::from(v))),
        ColumnData::F64(v) => v.map_or(Cell::Null, Cell::Number),
        ColumnData::Bit(v) => v.map_or(Cell::Null, Cell::Bool),
        ColumnData::String(v) => v.map_or(Cell::Null, |v| Cell::String(v.into_owned())),
        ColumnData::Guid(v) => v.map_or(Cell::Null, |v| Cell::String(v.to_string().to_uppercase())),
        ColumnData::Binary(v) => v.map_or(Cell::Null, |v| Cell::Buffer(v.into_owned())),
        ColumnData::Numeric(v) => v.map_or(Cell::Null, |v| {
            Cell::Number(v.value() as f64 / 10f64.powi(i32::from(v.scale())))
        }),
        ColumnData::Xml(v) => v.map_or(Cell::Null, |v| Cell::String(v.to_string())),
        ColumnData::DateTime(v) => v.map_or(Cell::Null, |v| {
            let ms = (f64::from(v.seconds_fragments()) * 10.0 / 3.0).round();
            Cell::Date((days_1900 + f64::from(v.days())) * MS_PER_DAY + ms)
        }),
        ColumnData::SmallDateTime(v) => v.map_or(Cell::Null, |v| {
            let ms = f64::from(v.seconds_fragments()) * 60_000.0;
            Cell::Date((days_1900 + f64::from(v.days())) * MS_PER_DAY + ms)
        }),
        ColumnData::Time(v) => v.map_or(Cell::Null, |v| Cell::Date(time_ms(v))),
        ColumnData::Date(v) => v.map_or(Cell::Null, |v| Cell::Date(date_ms(v))),
        ColumnData::DateTime2(v) => v.map_or(Cell::Null, |v| {
            Cell::Date(date_ms(v.date()) + time_ms(v.time()))
        }),
        ColumnData::DateTimeOffset(v) => v.map_or(Cell::Null, |v| {
            let utc = v.datetime2();
            Cell::Date(date_ms(utc.date()) + time_ms(utc.time()))
        }),
    }
}

fn text(row: &Row, name: &str) -> String {
    match row.get(name) {
        Some(Cell::String(value)) => value.clone(),
        _ => String::new(),
    }
}

fn get_tables(connection_string: &str) -> Result<Vec<Table>, Error> {
    block_on(async {
        let (mut client, timeout) = connect(connection_string).await?;
        let mut fetch = async |sql: &str| -> Result<Vec<Row>, Error> {
            let (sets, _) = execute(&mut client, timeout, sql).await?;
            Ok(sets.into_iter().next().unwrap_or_default())
        };
        let tables = fetch(TABLES_QUERY).await?;
        let columns = fetch(COLUMNS_QUERY).await?;
        let foreign_keys = fetch(FOREIGN_KEYS_QUERY).await?;
        let _ = client.close().await;

        Ok(tables
            .iter()
            .map(|table| {
                let schema = text(table, "table_schema");
                let name = text(table, "table_name");
                let columns = columns
                    .iter()
                    .filter(|c| text(c, "table_schema") == schema && text(c, "table_name") == name)
                    .map(|column| {
                        let column_name = text(column, "column_name");
                        let relation = foreign_keys
                            .iter()
                            .find(|fk| {
                                text(fk, "table_schema") == schema
                                    && text(fk, "table_name") == name
                                    && text(fk, "column_name") == column_name
                            })
                            .map(|fk| {
                                format!(
                                    "{}.{}.{}",
                                    text(fk, "foreign_table_schema"),
                                    text(fk, "foreign_table_name"),
                                    text(fk, "foreign_column_name")
                                )
                            });
                        Column {
                            name: column_name,
                            data_type: text(column, "data_type"),
                            nullable: text(column, "is_nullable") == "YES",
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

const TABLES_QUERY: &str = "SELECT table_schema, table_name FROM INFORMATION_SCHEMA.TABLES WHERE table_type = 'BASE TABLE' AND table_schema NOT IN ('sys', 'INFORMATION_SCHEMA') ORDER BY table_schema, table_name";

const COLUMNS_QUERY: &str = "SELECT table_schema, table_name, column_name, column_default, is_nullable, data_type FROM INFORMATION_SCHEMA.COLUMNS WHERE table_schema NOT IN ('sys', 'INFORMATION_SCHEMA') ORDER BY table_schema, table_name, ORDINAL_POSITION";

const FOREIGN_KEYS_QUERY: &str = "SELECT fk.TABLE_SCHEMA AS table_schema, fk.TABLE_NAME AS table_name, cu.COLUMN_NAME AS column_name, pk.TABLE_SCHEMA AS foreign_table_schema, pk.TABLE_NAME AS foreign_table_name, pt.COLUMN_NAME AS foreign_column_name FROM INFORMATION_SCHEMA.REFERENTIAL_CONSTRAINTS rc JOIN INFORMATION_SCHEMA.TABLE_CONSTRAINTS fk ON rc.CONSTRAINT_NAME = fk.CONSTRAINT_NAME AND rc.CONSTRAINT_SCHEMA = fk.CONSTRAINT_SCHEMA JOIN INFORMATION_SCHEMA.TABLE_CONSTRAINTS pk ON rc.UNIQUE_CONSTRAINT_NAME = pk.CONSTRAINT_NAME AND rc.UNIQUE_CONSTRAINT_SCHEMA = pk.CONSTRAINT_SCHEMA JOIN INFORMATION_SCHEMA.KEY_COLUMN_USAGE cu ON rc.CONSTRAINT_NAME = cu.CONSTRAINT_NAME AND rc.CONSTRAINT_SCHEMA = cu.CONSTRAINT_SCHEMA JOIN INFORMATION_SCHEMA.KEY_COLUMN_USAGE pt ON rc.UNIQUE_CONSTRAINT_NAME = pt.CONSTRAINT_NAME AND rc.UNIQUE_CONSTRAINT_SCHEMA = pt.CONSTRAINT_SCHEMA";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::to_iso_string;
    use tiberius::time::{Date, DateTime2, Time};

    #[test]
    fn converts_dates_like_tedious() {
        let date = Date::new((days_from_civil(2024, 1, 2) + days_since_year_one()) as u32);
        let time = Time::new(110_456_780_000, 7);
        let Cell::Date(ms) = cell(ColumnData::DateTime2(Some(DateTime2::new(date, time)))) else {
            panic!()
        };
        assert_eq!(to_iso_string(ms).unwrap(), "2024-01-02T03:04:05.678Z");
        let Cell::Date(ms) = cell(ColumnData::Time(Some(time))) else {
            panic!()
        };
        assert_eq!(to_iso_string(ms).unwrap(), "1970-01-01T03:04:05.678Z");
        let datetime =
            tiberius::time::DateTime::new(45291, 3 * 3600 * 300 + 4 * 60 * 300 + 5 * 300 + 203);
        let Cell::Date(ms) = cell(ColumnData::DateTime(Some(datetime))) else {
            panic!()
        };
        assert_eq!(to_iso_string(ms).unwrap(), "2024-01-02T03:04:05.677Z");
    }

    #[test]
    fn converts_numbers_like_tedious() {
        assert_eq!(
            cell(ColumnData::I64(Some(9007199254740993))),
            Cell::String("9007199254740993".into())
        );
        assert_eq!(
            cell(ColumnData::Numeric(Some(
                tiberius::numeric::Numeric::new_with_scale(12345, 3)
            ))),
            Cell::Number(12.345)
        );
        assert_eq!(cell(ColumnData::I32(None)), Cell::Null);
    }

    #[test]
    fn parses_connection_strings() {
        let (config, timeout) = parse_connection_string(
            "mssql://sa:World123!%3B@localhost:1433/db?encrypt=false&trustServerCertificate=true",
        )
        .unwrap();
        assert_eq!(config.get_addr(), "localhost:1433");
        assert_eq!(timeout, Duration::from_millis(10000));
    }
}
