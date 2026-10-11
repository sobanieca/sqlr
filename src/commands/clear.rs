use crate::connection;
use crate::logger;
use crate::scope;
use crate::storage::Storage;

pub fn execute(is_global: bool) -> Result<(), String> {
    let mut storage = Storage::load()?;
    connection::clear(&mut storage, is_global)?;
    scope::clear_default_connection(&mut storage, is_global)?;
    logger::info("All connections removed");
    Ok(())
}
