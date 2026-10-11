//! Stored connections: reading, decrypting, selecting and styling them.

use crate::colors;
use crate::guard;
use crate::json::{Object, Value};
use crate::logger;
use crate::prompt;
use crate::scope;
use crate::storage::Storage;

pub const ENCRYPTION_PASSWORD_ENV: &str = "SQLR_ENCRYPTION_PASSWORD";

#[derive(Clone, Debug, PartialEq)]
pub struct Connection {
    pub name: String,
    pub connection_type: String,
    pub connection_string: String,
    pub color: String,
    pub emoji: String,
    pub is_encrypted: bool,
}

impl Connection {
    pub fn to_value(&self) -> Value {
        let mut object = Object::new();
        object.insert("name", self.name.as_str().into());
        object.insert("type", self.connection_type.as_str().into());
        object.insert("connectionString", self.connection_string.as_str().into());
        object.insert("color", self.color.as_str().into());
        object.insert("emoji", self.emoji.as_str().into());
        object.insert("isEncrypted", self.is_encrypted.into());
        Value::Object(object)
    }

    fn from_value(value: &Value) -> Option<Connection> {
        let object = value.as_object()?;
        let text = |key: &str| object.get_str(key).unwrap_or_default().to_string();
        Some(Connection {
            name: object.get_str("name")?.to_string(),
            connection_type: text("type"),
            connection_string: text("connectionString"),
            color: text("color"),
            emoji: text("emoji"),
            is_encrypted: object.is_truthy("isEncrypted"),
        })
    }
}

pub fn exists(storage: &Storage, name: &str, is_global: bool) -> bool {
    let key = format!("{}{name}", scope::scope_prefix(storage, is_global));
    storage.get(&key).is_some()
}

pub fn save(storage: &mut Storage, connection: &Connection, is_global: bool) -> Result<(), String> {
    let key = format!("{}{}", scope::scope_prefix(storage, is_global), connection.name);
    storage.set(&key, connection.to_value())
}

pub fn remove(storage: &mut Storage, name: &str, is_global: bool) -> Result<(), String> {
    let key = format!("{}{name}", scope::scope_prefix(storage, is_global));
    storage.remove(&key)
}

pub fn clear(storage: &mut Storage, is_global: bool) -> Result<(), String> {
    let prefix = scope::scope_prefix(storage, is_global);
    storage.remove_where(|key| key.starts_with(&prefix))
}

/// Connections in the effective scope, in the order they were added.
pub fn all(storage: &Storage, is_global: bool) -> Vec<Connection> {
    let prefix = scope::scope_prefix(storage, is_global);
    storage
        .keys()
        .filter(|key| key.starts_with(&prefix))
        .filter_map(|key| storage.get(key).and_then(Connection::from_value))
        .collect()
}

/// Reads a connection and decrypts its connection string, asking for the
/// password unless `SQLR_ENCRYPTION_PASSWORD` is set.
pub fn get(storage: &Storage, name: &str, is_global: bool) -> Result<Connection, String> {
    let key = format!("{}{name}", scope::scope_prefix(storage, is_global));
    let Some(value) = storage.get(&key) else {
        return Err(format!(
            "Connection {name} not found. Use ls command to list available connections."
        ));
    };
    let mut connection = Connection::from_value(value).ok_or_else(|| {
        logger::debug(format!("Error when reading connection {name}"));
        "Unknown error occured when reading connection. Enable debug mode for details."
            .to_string()
    })?;

    if connection.is_encrypted {
        let password = match std::env::var(ENCRYPTION_PASSWORD_ENV) {
            Ok(password) if !password.is_empty() => password,
            _ => prompt::secret("Provide password used to encrypt connection")?,
        };
        connection.connection_string = guard::decrypt(&connection.connection_string, &password)
            .map_err(|err| {
                logger::debug(err);
                format!(
                    "Something went wrong when trying to decrypt connection {name}. Ensure that password you've provided is valid"
                )
            })?;
    }
    Ok(connection)
}

/// The default connection of the scope, otherwise asks the user to pick one.
pub fn select_name(storage: &Storage, is_global: bool) -> Result<String, String> {
    if let Some(default) = scope::default_connection(storage, is_global)
        && exists(storage, &default, is_global)
    {
        return Ok(default);
    }
    let options: Vec<(String, String)> = all(storage, is_global)
        .into_iter()
        .map(|connection| (connection.name.clone(), connection.name))
        .collect();
    if options.is_empty() {
        return Err("No connections defined. Add new connection first.".to_string());
    }
    prompt::select("Select connection", &options)
}

pub struct Color {
    pub name: &'static str,
    pub value: &'static str,
    pub rgb: u32,
}

pub const COLORS: [Color; 10] = [
    Color { name: "Red", value: "red", rgb: 0xff0000 },
    Color { name: "Dark Red", value: "dark-red", rgb: 0x8b0000 },
    Color { name: "Yellow", value: "yellow", rgb: 0xffd700 },
    Color { name: "Dark Yellow", value: "dark-yellow", rgb: 0xb8860b },
    Color { name: "Gray", value: "gray", rgb: 0xa9a9a9 },
    Color { name: "Dark Gray", value: "dark-gray", rgb: 0x696969 },
    Color { name: "Blue", value: "blue", rgb: 0x5b9bd5 },
    Color { name: "Dark Blue", value: "dark-blue", rgb: 0x00008b },
    Color { name: "Orange", value: "orange", rgb: 0xff8c00 },
    Color { name: "Dark Orange", value: "dark-orange", rgb: 0xd2691e },
];

const EMOJIS: [(&str, &str); 10] = [
    ("None", "none"),
    ("\u{1F680} Rocket", "\u{1F680}"),
    ("\u{1F6A7} Construction", "\u{1F6A7}"),
    ("\u{1F9EA} Laboratory", "\u{1F9EA}"),
    ("\u{2699}\u{FE0F} Gear", "\u{2699}\u{FE0F}"),
    ("\u{1F30D} Globe", "\u{1F30D}"),
    ("\u{1F512} Lock", "\u{1F512}"),
    ("\u{26A1} Lightning", "\u{26A1}"),
    ("\u{1F4E6} Package", "\u{1F4E6}"),
    ("Custom", "custom"),
];

/// Color names accepted by `add --color`.
pub fn color_values() -> Vec<String> {
    COLORS
        .iter()
        .map(|color| color.value.to_string())
        .chain(["none".to_string()])
        .collect()
}

pub fn prompt_color() -> Result<String, String> {
    let options: Vec<(String, String)> = std::iter::once(("None".to_string(), "none".to_string()))
        .chain(COLORS.iter().map(|color| {
            (
                format!("{} {}", colors::rgb24("\u{2588}\u{2588}", color.rgb), color.name),
                color.value.to_string(),
            )
        }))
        .collect();
    prompt::select("Select connection color", &options)
}

pub fn prompt_emoji() -> Result<String, String> {
    let options: Vec<(String, String)> = EMOJIS
        .iter()
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .collect();
    let choice = prompt::select("Select connection emoji", &options)?;
    if choice == "custom" {
        return prompt::input("Enter custom emoji");
    }
    Ok(choice)
}

/// Connection name with its emoji and color.
pub fn styled_name(connection: &Connection) -> String {
    let mut name = connection.name.clone();
    if let Some(color) = COLORS.iter().find(|color| color.value == connection.color) {
        name = colors::rgb24(&name, color.rgb);
    }
    if !connection.emoji.is_empty() && connection.emoji != "none" {
        name = format!("{} {name}", connection.emoji);
    }
    name
}

pub fn format_connection_name(connection: &Connection) -> String {
    format!(
        "{} {}",
        colors::gray("Using connection:"),
        styled_name(connection)
    )
}
