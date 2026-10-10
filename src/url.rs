//! Minimal URL parsing for connection strings, following the parts of the
//! WHATWG `URL` behavior the connectors rely on: user info, host, port, path
//! and `URLSearchParams` style query parameters.

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Url {
    /// Lowercase scheme without `:` (e.g. `postgres`).
    pub scheme: String,
    /// Percent-encoded user name, as `URL.username` returns it.
    pub username: String,
    /// Percent-encoded password, as `URL.password` returns it.
    pub password: String,
    /// Host name without port. IPv6 addresses keep their brackets.
    pub host: String,
    pub port: Option<u16>,
    /// Path including the leading `/` (may be empty).
    pub path: String,
    /// Decoded query parameters in order.
    pub query: Vec<(String, String)>,
}

impl Url {
    pub fn parse(input: &str) -> Result<Url, String> {
        let invalid = || format!("Invalid URL: '{input}'");
        let input = input.trim();
        let (scheme, rest) = input.split_once(':').ok_or_else(invalid)?;
        if scheme.is_empty()
            || !scheme.starts_with(|c: char| c.is_ascii_alphabetic())
            || !scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
        {
            return Err(invalid());
        }
        let rest = rest.strip_prefix("//").ok_or_else(invalid)?;
        let rest = rest.split('#').next().unwrap_or_default();
        let authority_end = rest.find(['/', '?']).unwrap_or(rest.len());
        let (authority, tail) = rest.split_at(authority_end);
        let (path, query) = match tail.split_once('?') {
            Some((path, query)) => (path, Some(query)),
            None => (tail, None),
        };

        let (user_info, host_port) = match authority.rsplit_once('@') {
            Some((user_info, host_port)) => (Some(user_info), host_port),
            None => (None, authority),
        };
        let (username, password) = match user_info {
            Some(info) => match info.split_once(':') {
                Some((user, password)) => (user, password),
                None => (info, ""),
            },
            None => ("", ""),
        };

        let (host, port) = if host_port.starts_with('[') {
            let end = host_port.find(']').ok_or_else(invalid)?;
            let host = &host_port[..=end];
            let port = host_port[end + 1..].strip_prefix(':');
            (host, port)
        } else {
            match host_port.rsplit_once(':') {
                Some((host, port)) => (host, Some(port)),
                None => (host_port, None),
            }
        };
        let port = match port {
            Some("") | None => None,
            Some(port) => Some(port.parse::<u16>().map_err(|_| invalid())?),
        };
        let scheme = scheme.to_ascii_lowercase();
        if host.is_empty() && matches!(scheme.as_str(), "http" | "https") {
            return Err(invalid());
        }
        let port = match (scheme.as_str(), port) {
            ("http", Some(80)) | ("https", Some(443)) => None,
            (_, port) => port,
        };

        Ok(Url {
            username: username.to_string(),
            password: password.to_string(),
            host: host.to_ascii_lowercase(),
            port,
            path: path.to_string(),
            query: query.map(parse_query).unwrap_or_default(),
            scheme,
        })
    }

    /// `searchParams.get(name)`: the first value of the parameter.
    pub fn param(&self, name: &str) -> Option<&str> {
        self.query
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    /// `searchParams.delete(name)`
    pub fn remove_param(&mut self, name: &str) {
        self.query.retain(|(key, _)| key != name);
    }

    /// Path without the leading `/`, as `pathname.replace("/", "")`.
    pub fn path_name(&self) -> String {
        self.path.replacen('/', "", 1)
    }

    /// Decoded user name.
    pub fn user(&self) -> String {
        percent_decode(&self.username)
    }

    /// Decoded password.
    pub fn decoded_password(&self) -> String {
        percent_decode(&self.password)
    }

    /// `host:port` or `host`.
    pub fn host_with_port(&self) -> String {
        match self.port {
            Some(port) => format!("{}:{port}", self.host),
            None => self.host.clone(),
        }
    }

    /// Host without IPv6 brackets.
    pub fn hostname(&self) -> &str {
        self.host.trim_start_matches('[').trim_end_matches(']')
    }
}

/// Parses `application/x-www-form-urlencoded` query strings like
/// `URLSearchParams`: `+` is a space and values are percent-decoded.
pub fn parse_query(query: &str) -> Vec<(String, String)> {
    query
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            (
                percent_decode(&key.replace('+', " ")),
                percent_decode(&value.replace('+', " ")),
            )
        })
        .collect()
}

/// Decodes `%XX` sequences, leaving invalid sequences unchanged. Invalid
/// UTF-8 is replaced like `TextDecoder` does.
pub fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let (Some(high), Some(low)) = (hex(bytes[i + 1]), hex(bytes[i + 2]))
        {
            out.push(high * 16 + low);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `decodeURIComponent`: fails on malformed sequences.
pub fn decode_uri_component(s: &str) -> Result<String, String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let (Some(high), Some(low)) = (
                bytes.get(i + 1).copied().and_then(hex),
                bytes.get(i + 2).copied().and_then(hex),
            ) else {
                return Err("URI malformed".to_string());
            };
            out.push(high * 16 + low);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).map_err(|_| "URI malformed".to_string())
}

/// `encodeURIComponent`
pub fn encode_uri_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for byte in s.bytes() {
        if byte.is_ascii_alphanumeric()
            || matches!(
                byte,
                b'-' | b'_' | b'.' | b'!' | b'~' | b'*' | b'\'' | b'(' | b')'
            )
        {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_connection_urls() {
        let url = Url::parse(
            "postgres://localhost:5432/world-db?user=world&password=p%40ss&application_name=sqlr",
        )
        .unwrap();
        assert_eq!(url.scheme, "postgres");
        assert_eq!(url.host, "localhost");
        assert_eq!(url.port, Some(5432));
        assert_eq!(url.path_name(), "world-db");
        assert_eq!(url.param("password"), Some("p@ss"));
        assert_eq!(url.param("missing"), None);

        let url = Url::parse("mssql://sa:World123!%3B@localhost:1433/db?encrypt=false").unwrap();
        assert_eq!(url.username, "sa");
        assert_eq!(url.password, "World123!%3B");
        assert_eq!(url.decoded_password(), "World123!;");
        assert_eq!(url.param("encrypt"), Some("false"));

        let url = Url::parse("http://world:world123@localhost:8123?database=world").unwrap();
        assert_eq!(url.path, "");
        assert_eq!(url.host_with_port(), "localhost:8123");

        let url = Url::parse("https://u:p@[::1]:8443/db").unwrap();
        assert_eq!(url.host, "[::1]");
        assert_eq!(url.hostname(), "::1");
        assert_eq!(url.port, Some(8443));

        assert!(Url::parse("/tmp/db.sqlite").is_err());
        assert!(Url::parse("postgres://host:notaport/db").is_err());
    }

    #[test]
    fn encodes_and_decodes_components() {
        assert_eq!(encode_uri_component("p@ss:w#rd;!*"), "p%40ss%3Aw%23rd%3B!*");
        assert_eq!(decode_uri_component("p%40ss%3Aw%23rd").unwrap(), "p@ss:w#rd");
        assert!(decode_uri_component("100%").is_err());
        assert_eq!(percent_decode("100%"), "100%");
        assert_eq!(parse_query("a=1+2&b=%7B%7D&c"), [
            ("a".to_string(), "1 2".to_string()),
            ("b".to_string(), "{}".to_string()),
            ("c".to_string(), String::new()),
        ]);
    }
}
