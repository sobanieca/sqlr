//! Persistent key-value storage for connections and settings (replaces the
//! `localStorage` used by the Deno version). Data is kept in
//! `$XDG_CONFIG_HOME/sqlr/storage.json` (default `~/.config/sqlr`), readable
//! only by the current user.

use crate::json::{self, Object, Value};
use std::path::PathBuf;

const FILE_NAME: &str = "storage.json";

pub fn home_dir() -> Option<String> {
    std::env::var("HOME")
        .ok()
        .or_else(|| std::env::var("USERPROFILE").ok())
        .filter(|home| !home.is_empty())
}

fn storage_dir() -> Result<PathBuf, String> {
    if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME").filter(|dir| !dir.is_empty()) {
        return Ok(PathBuf::from(dir).join("sqlr"));
    }
    let home = home_dir().ok_or("Unable to locate the home directory (set HOME)")?;
    Ok(PathBuf::from(home).join(".config").join("sqlr"))
}

fn storage_file() -> Result<PathBuf, String> {
    Ok(storage_dir()?.join(FILE_NAME))
}

pub struct Storage {
    data: Object,
}

impl Storage {
    pub fn load() -> Result<Storage, String> {
        let path = storage_file()?;
        let data = match std::fs::read_to_string(&path) {
            Ok(content) => match json::parse(&content) {
                Ok(Value::Object(data)) => data,
                _ => return Err(format!("{} is corrupted", path.display())),
            },
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Object::new(),
            Err(err) => return Err(format!("Failed to read {}: {err}", path.display())),
        };
        Ok(Storage { data })
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.data.keys()
    }

    pub fn set(&mut self, key: &str, value: Value) -> Result<(), String> {
        self.data.insert(key, value);
        self.save()
    }

    pub fn remove(&mut self, key: &str) -> Result<(), String> {
        self.remove_where(|k| k == key)
    }

    pub fn remove_where(&mut self, predicate: impl Fn(&str) -> bool) -> Result<(), String> {
        let before = self.data.len();
        self.data = self
            .data
            .iter()
            .filter(|(key, _)| !predicate(key))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        if self.data.len() == before {
            return Ok(());
        }
        self.save()
    }

    fn save(&self) -> Result<(), String> {
        let dir = storage_dir()?;
        let path = dir.join(FILE_NAME);
        let error = |err: std::io::Error| format!("Failed to write {}: {err}", path.display());
        create_private_dir(&dir).map_err(error)?;
        let temp = dir.join(format!(".{FILE_NAME}-{}", std::process::id()));
        write_private_file(
            &temp,
            json::stringify(&Value::Object(self.data.clone())).as_bytes(),
        )
        .map_err(error)?;
        std::fs::rename(&temp, &path).map_err(|err| {
            let _ = std::fs::remove_file(&temp);
            error(err)
        })
    }
}

#[cfg(unix)]
fn create_private_dir(dir: &std::path::Path) -> std::io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(dir)
}

#[cfg(not(unix))]
fn create_private_dir(dir: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)
}

#[cfg(unix)]
fn write_private_file(path: &std::path::Path, content: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(content)
}

#[cfg(not(unix))]
fn write_private_file(path: &std::path::Path, content: &[u8]) -> std::io::Result<()> {
    std::fs::write(path, content)
}
