use crate::connection;
use crate::logger;
use crate::scope;
use crate::storage::Storage;
use crate::table::Table;

pub fn execute(is_global: bool) -> Result<(), String> {
    let storage = Storage::load()?;
    let default = scope::default_connection(&storage, is_global);
    let rows: Vec<Vec<String>> = connection::all(&storage, is_global)
        .iter()
        .map(|connection| {
            let marker = if default.as_deref() == Some(connection.name.as_str()) {
                " [default]"
            } else {
                ""
            };
            vec![
                format!("{}{marker}", connection::styled_name(connection)),
                connection.connection_type.clone(),
            ]
        })
        .collect();

    let table = Table::new()
        .header(vec!["Name".to_string(), "Type".to_string()])
        .body(rows)
        .max_col_width(25)
        .padding(1)
        .indent(2)
        .border(true)
        .to_string();

    if scope::is_global_mode_enabled(&storage) {
        logger::info("Global mode: enabled");
    }
    logger::info(table);
    Ok(())
}
