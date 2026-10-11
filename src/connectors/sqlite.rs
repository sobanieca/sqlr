//! SQLite connector. The connection string is the database file path.

use super::{Column, Connector, Error, QueryResult, Table};
use crate::prompt;
use crate::value::{Cell, Row};
use rusqlite::types::ValueRef;
use rusqlite::{Connection, OpenFlags};

pub const CONNECTOR: Connector = Connector {
    key: "sqlite",
    database_name: "SQLite",
    connection_string_hint,
    prompt_connection_string,
    get_tables,
    query,
};

fn connection_string_hint() -> String {
    "/path/to/database.db\nMore details: https://www.sqlite.org/uri.html".to_string()
}

fn prompt_connection_string() -> Result<String, String> {
    prompt::input("Database file path (example: /data/mydb.sqlite)")
}

fn database_error(err: rusqlite::Error) -> Error {
    let message = match &err {
        rusqlite::Error::SqliteFailure(_, Some(message))
        | rusqlite::Error::SqlInputError { msg: message, .. } => message.clone(),
        other => other.to_string(),
    };
    Error::Database(format!("SQLite Error: {message}"))
}

fn is_select_query(sql: &str) -> bool {
    let sql = sql.trim().to_uppercase();
    ["SELECT", "WITH", "PRAGMA", "EXPLAIN"]
        .iter()
        .any(|keyword| sql.starts_with(keyword))
}

fn get_tables(connection_string: &str) -> Result<Vec<Table>, Error> {
    let db = Connection::open_with_flags(
        connection_string,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )
    .map_err(database_error)?;
    let names: Vec<String> = db
        .prepare(
            "SELECT name AS table_name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )
        .and_then(|mut statement| {
            statement
                .query_map([], |row| row.get::<_, String>(0))?
                .collect()
        })
        .map_err(database_error)?;

    names
        .into_iter()
        .map(|name| {
            let quoted = name.replace('\'', "''");
            let foreign_keys: Vec<(String, String, Option<String>)> = db
                .prepare(&format!("PRAGMA foreign_key_list('{quoted}')"))
                .and_then(|mut statement| {
                    statement
                        .query_map([], |row| {
                            Ok((row.get("from")?, row.get("table")?, row.get("to")?))
                        })?
                        .collect()
                })
                .map_err(database_error)?;
            let columns = db
                .prepare(&format!("PRAGMA table_info('{quoted}')"))
                .and_then(|mut statement| {
                    statement
                        .query_map([], |row| {
                            let name: String = row.get("name")?;
                            let data_type: String = row.get("type")?;
                            let not_null: i64 = row.get("notnull")?;
                            let pk: i64 = row.get("pk")?;
                            let relation = foreign_keys
                                .iter()
                                .find(|(from, _, _)| *from == name)
                                .map(|(_, table, to)| {
                                    format!("main.{table}.{}", to.as_deref().unwrap_or("null"))
                                });
                            Ok(Column {
                                name,
                                data_type: if data_type.is_empty() {
                                    "ANY".to_string()
                                } else {
                                    data_type
                                },
                                nullable: not_null == 0 && pk == 0,
                                relation,
                            })
                        })?
                        .collect()
                })
                .map_err(database_error)?;
            Ok(Table {
                schema: "main".to_string(),
                name,
                columns,
            })
        })
        .collect()
}

fn cell(value: ValueRef) -> Cell {
    match value {
        ValueRef::Null => Cell::Null,
        ValueRef::Integer(n) => Cell::Number(n as f64),
        ValueRef::Real(n) => Cell::Number(n),
        ValueRef::Text(text) => Cell::String(String::from_utf8_lossy(text).into_owned()),
        ValueRef::Blob(bytes) => Cell::Uint8Array(bytes.to_vec()),
    }
}

fn query(connection_string: &str, sql: &str) -> Result<QueryResult, Error> {
    let db = Connection::open(connection_string).map_err(database_error)?;

    if !is_select_query(sql) {
        let before = db.total_changes();
        db.execute_batch(sql).map_err(database_error)?;
        return Ok(QueryResult {
            rows_affected: Cell::Number((db.total_changes() - before) as f64),
            rows: Vec::new(),
        });
    }

    let mut statement = db.prepare(sql).map_err(database_error)?;
    let names: Vec<String> = statement
        .column_names()
        .iter()
        .map(|name| name.to_string())
        .collect();
    let mut rows = statement.query([]).map_err(database_error)?;
    let mut result = Vec::new();
    while let Some(row) = rows.next().map_err(database_error)? {
        let mut object = Row::new();
        for (index, name) in names.iter().enumerate() {
            let value = row.get_ref(index).map_err(database_error)?;
            object.insert(name.clone(), cell(value));
        }
        result.push(object);
    }
    Ok(QueryResult {
        rows_affected: Cell::Number(result.len() as f64),
        rows: result,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queries_and_counts_changes() {
        let path = std::env::temp_dir().join(format!("sqlr-sqlite-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let cs = path.to_str().unwrap();
        let created = query(cs, "CREATE TABLE t (id INTEGER, b BLOB); INSERT INTO t VALUES (1, x'01'), (2, NULL)").unwrap();
        assert_eq!(created.rows_affected, Cell::Number(2.0));
        let updated = query(cs, "UPDATE t SET id = id + 1").unwrap();
        assert_eq!(updated.rows_affected, Cell::Number(2.0));
        let selected = query(cs, "  select id, b, 1.5 AS r, 'x' AS s FROM t ORDER BY id").unwrap();
        assert_eq!(selected.rows_affected, Cell::Number(2.0));
        assert_eq!(selected.rows[0].get("b"), Some(&Cell::Uint8Array(vec![1])));
        assert_eq!(selected.rows[1].get("b"), Some(&Cell::Null));
        assert_eq!(selected.rows[0].get("r"), Some(&Cell::Number(1.5)));
        assert_eq!(
            query(cs, "INVALID SQL QUERY").unwrap_err(),
            Error::Database("SQLite Error: near \"INVALID\": syntax error".to_string())
        );
        let tables = get_tables(cs).unwrap();
        assert_eq!(tables[0].name, "t");
        assert_eq!(tables[0].columns[1].data_type, "BLOB");
        let _ = std::fs::remove_file(&path);
    }
}
