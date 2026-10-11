use crate::connection;
use crate::logger;
use crate::scope;
use crate::storage::Storage;

/// `name_flag` is set when `-n` was given (with or without a value).
pub fn execute(name: Option<String>, name_flag: bool, is_global: bool) -> Result<(), String> {
    let mut storage = Storage::load()?;
    if is_global {
        scope::set_global_mode(&mut storage, true)?;
    }

    let name = match name {
        Some(name) => name,
        None if !name_flag => {
            scope::clear_default_connection(&mut storage, is_global)?;
            logger::info("Default connection has been cleared");
            return Ok(());
        }
        None => connection::select_name(&storage, is_global)?,
    };

    if !connection::exists(&storage, &name, is_global) {
        return Err(format!(
            "Connection {name} not found. Use ls command to list available connections."
        ));
    }
    scope::set_default_connection(&mut storage, &name, is_global)?;
    logger::info(format!("Default connection set to {name}"));
    Ok(())
}
