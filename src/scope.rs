//! Connection scopes. Connections belong to the nearest git repository
//! (searched from the current directory up to the home directory) or to the
//! global scope. Global mode makes every command use the global scope.

use crate::logger;
use crate::storage::{Storage, home_dir};
use std::path::Path;

const GLOBAL: &str = "global";
const GLOBAL_MODE_KEY: &str = "sqlr:global-mode";

/// The git repository root containing the current directory, or "global".
pub fn resolve_scope() -> String {
    let home = home_dir();
    let mut dir = std::env::current_dir().unwrap_or_else(|_| "/".into());
    loop {
        if dir.join(".git").exists() {
            return dir.to_string_lossy().into_owned();
        }
        if home.as_deref().map(Path::new) == Some(dir.as_path()) {
            return GLOBAL.to_string();
        }
        if !dir.pop() {
            return GLOBAL.to_string();
        }
    }
}

pub fn is_global_mode_enabled(storage: &Storage) -> bool {
    storage.get(GLOBAL_MODE_KEY).and_then(|v| v.as_str()) == Some("true")
}

pub fn set_global_mode(storage: &mut Storage, enabled: bool) -> Result<(), String> {
    logger::debug(format!("Setting global mode to {enabled}"));
    if enabled {
        storage.set(GLOBAL_MODE_KEY, "true".into())
    } else {
        storage.remove(GLOBAL_MODE_KEY)
    }
}

fn effective_scope(storage: &Storage, is_global: bool) -> String {
    if is_global || is_global_mode_enabled(storage) {
        GLOBAL.to_string()
    } else {
        resolve_scope()
    }
}

/// Prefix of connection keys in the effective scope.
pub fn scope_prefix(storage: &Storage, is_global: bool) -> String {
    format!("scope:{}:", effective_scope(storage, is_global))
}

fn default_connection_key(storage: &Storage, is_global: bool) -> String {
    format!(
        "sqlr:default-connection:{}",
        effective_scope(storage, is_global)
    )
}

pub fn default_connection(storage: &Storage, is_global: bool) -> Option<String> {
    storage
        .get(&default_connection_key(storage, is_global))
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

pub fn set_default_connection(
    storage: &mut Storage,
    name: &str,
    is_global: bool,
) -> Result<(), String> {
    logger::debug(format!(
        "Setting default connection to '{name}' (global: {is_global})"
    ));
    let key = default_connection_key(storage, is_global);
    storage.set(&key, name.into())
}

pub fn clear_default_connection(storage: &mut Storage, is_global: bool) -> Result<(), String> {
    logger::debug(format!(
        "Clearing default connection (global: {is_global})"
    ));
    let key = default_connection_key(storage, is_global);
    storage.remove(&key)
}
