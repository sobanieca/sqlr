use crate::connection::{self, Connection, ENCRYPTION_PASSWORD_ENV};
use crate::connectors;
use crate::guard;
use crate::logger;
use crate::prompt;
use crate::storage::Storage;

const MIN_PASSWORD_LENGTH: usize = 8;

pub struct Options {
    pub name: Option<String>,
    pub connection_type: Option<String>,
    pub connection_string: Option<String>,
    pub color: Option<String>,
    pub emoji: Option<String>,
    pub is_global: bool,
}

/// `^[A-Za-z0-9]+-*[A-Za-z0-9]+$`
fn is_valid_name(name: &str) -> bool {
    let dash_runs = name
        .split(|c: char| c.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .count();
    name.len() >= 2
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-')
        && dash_runs <= 1
}

fn prompt_connection_type() -> Result<String, String> {
    let options: Vec<(String, String)> = connectors::CONNECTORS
        .iter()
        .map(|c| (c.database_name.to_string(), c.key.to_string()))
        .collect();
    prompt::select("Select database type", &options)
}

pub fn execute(options: Options) -> Result<(), String> {
    let mut storage = Storage::load()?;
    let name = match options.name {
        Some(name) => name,
        None => prompt::input("Provide name of the connection")?,
    };
    if !is_valid_name(&name) {
        return Err(
            "Invalid connection name, please use only alphanumeric characters and/or at most 1 '-'"
                .to_string(),
        );
    }
    if connection::exists(&storage, &name, options.is_global) {
        return Err("Invalid connection name. Such connection already exists.".to_string());
    }

    let connection_type = match options.connection_type {
        Some(connection_type) => connection_type,
        None => prompt_connection_type()?,
    };
    let connector = connectors::find(&connection_type)
        .ok_or_else(|| format!("Unknown connection type '{connection_type}'"))?;
    let interactive = options.connection_string.is_none();
    let connection_string = match options.connection_string {
        Some(connection_string) => connection_string,
        None => {
            logger::info("Provide connection details");
            (connector.prompt_connection_string)()?
        }
    };

    let color = match options.color {
        Some(color) => color,
        None if interactive => connection::prompt_color()?,
        None => "none".to_string(),
    };
    let emoji = match options.emoji {
        Some(emoji) => emoji,
        None if interactive => connection::prompt_emoji()?,
        None => "none".to_string(),
    };

    let env_password = std::env::var(ENCRYPTION_PASSWORD_ENV)
        .ok()
        .filter(|password| !password.is_empty());
    let is_encrypted = if !interactive {
        env_password.is_some()
    } else {
        prompt::toggle(
            "Do you want to additionally secure connection settings? (Use for Production connections, each time when connection will be used you will need to specify password.",
        )?
    };

    let mut connection = Connection {
        name,
        connection_type,
        connection_string,
        color,
        emoji,
        is_encrypted,
    };

    if is_encrypted {
        let password = match env_password {
            Some(password) => password,
            None => prompt::secret("Provide password to encrypt connection")?,
        };
        if password.chars().count() < MIN_PASSWORD_LENGTH {
            return Err(format!(
                "Please provide password with at least {MIN_PASSWORD_LENGTH} characters"
            ));
        }
        connection.connection_string = guard::encrypt(&connection.connection_string, &password)?;
    }

    connection::save(&mut storage, &connection, options.is_global)?;
    logger::info("Connection has been added");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_names() {
        for name in ["ab", "prod-db", "a-b", "db1", "a--b"] {
            assert!(is_valid_name(name), "{name}");
        }
        for name in ["a", "-ab", "ab-", "a-b-c", "a_b", "", "a b", "zażółć"] {
            assert!(!is_valid_name(name), "{name}");
        }
    }
}
