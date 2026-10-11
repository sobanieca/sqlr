use crate::connection;
use crate::logger;
use crate::scope;
use crate::storage::Storage;

pub fn execute(name: Option<String>, is_global: bool) -> Result<(), String> {
    let mut storage = Storage::load()?;
    let name = match name {
        Some(name) => name,
        None => connection::select_name(&storage, is_global)?,
    };
    connection::remove(&mut storage, &name, is_global)?;
    if scope::default_connection(&storage, is_global).as_deref() == Some(name.as_str()) {
        scope::clear_default_connection(&mut storage, is_global)?;
    }
    logger::info("Connection removed");
    Ok(())
}
