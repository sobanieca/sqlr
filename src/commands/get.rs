use super::MAX_TABLE_COLUMN_WIDTH;
use crate::connection;
use crate::logger;
use crate::storage::Storage;
use crate::table::Table;
use crate::url::decode_uri_component;

pub fn execute(name: Option<String>, is_global: bool) -> Result<(), String> {
    let storage = Storage::load()?;
    let name = match name {
        Some(name) => name,
        None => connection::select_name(&storage, is_global)?,
    };
    let connection = connection::get(&storage, &name, is_global)?;
    let table = Table::new()
        .header(vec![
            "Name".to_string(),
            "Type".to_string(),
            "Connection string".to_string(),
        ])
        .body(vec![vec![
            connection::styled_name(&connection),
            connection.connection_type.clone(),
            decode_uri_component(&connection.connection_string)?,
        ]])
        .max_col_width(MAX_TABLE_COLUMN_WIDTH)
        .padding(1)
        .indent(2)
        .border(true)
        .to_string();
    logger::info(table);
    Ok(())
}
