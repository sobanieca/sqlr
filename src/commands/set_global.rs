use crate::logger;
use crate::scope;
use crate::storage::Storage;

pub fn execute(enabled: bool) -> Result<(), String> {
    let mut storage = Storage::load()?;
    scope::set_global_mode(&mut storage, enabled)?;
    if enabled {
        logger::info(
            "Global mode enabled. All connection commands will use global scope by default.",
        );
    } else {
        logger::info("Global mode disabled. Connection commands will use repository scope.");
    }
    Ok(())
}
