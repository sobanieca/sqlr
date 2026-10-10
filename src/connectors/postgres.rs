//! PostgreSQL connector.

use super::{Connector, Error, QueryResult, Table};

pub const CONNECTOR: Connector = Connector {
    key: "postgresql",
    database_name: "PostgreSQL",
    connection_string_hint,
    prompt_connection_string,
    get_tables,
    query,
};

fn connection_string_hint() -> String {
    todo!()
}

fn prompt_connection_string() -> Result<String, String> {
    todo!()
}

fn get_tables(_connection_string: &str) -> Result<Vec<Table>, Error> {
    todo!()
}

fn query(_connection_string: &str, _sql: &str) -> Result<QueryResult, Error> {
    todo!()
}
