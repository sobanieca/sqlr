//! Database connectors. Each connector parses its connection string format,
//! runs queries and reads the schema. Values are converted to [`Cell`]s the
//! way the Node.js driver used by the Deno version converted them.
//!
//! [`Cell`]: crate::value::Cell

pub mod clickhouse;
pub mod mssql;
pub mod mysql;
pub mod postgres;
pub mod sqlite;

use crate::value::{Cell, Row};
use std::future::Future;
use std::sync::Arc;

pub const DEFAULT_CONNECTION_TIMEOUT_MS: u64 = 10000;

#[derive(Clone, Debug, PartialEq)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    /// Foreign key target as `schema.table.column`.
    pub relation: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    pub schema: String,
    pub name: String,
    pub columns: Vec<Column>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct QueryResult {
    /// Usually a number. PostgreSQL reports `null` for statements without a
    /// row count and `undefined` for multiple statements.
    pub rows_affected: Cell,
    pub rows: Vec<Row>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// An error reported by the database (`DatabaseError` in the Deno
    /// version). The message is shown to the user as is.
    Database(String),
    /// Any other failure (invalid connection string, connection refused...).
    /// Only shown with `--debug`.
    Other(String),
}

impl Error {
    pub fn other(err: impl std::fmt::Display) -> Self {
        Error::Other(err.to_string())
    }
}

pub struct Connector {
    /// Type used on the command line and in stored connections.
    pub key: &'static str,
    pub database_name: &'static str,
    pub connection_string_hint: fn() -> String,
    /// Interactive wizard asking for connection details.
    pub prompt_connection_string: fn() -> Result<String, String>,
    pub get_tables: fn(&str) -> Result<Vec<Table>, Error>,
    pub query: fn(&str, &str) -> Result<QueryResult, Error>,
}

pub const CONNECTORS: [&Connector; 5] = [
    &postgres::CONNECTOR,
    &mysql::CONNECTOR,
    &mssql::CONNECTOR,
    &clickhouse::CONNECTOR,
    &sqlite::CONNECTOR,
];

pub fn find(key: &str) -> Option<&'static Connector> {
    CONNECTORS.into_iter().find(|connector| connector.key == key)
}

pub fn keys() -> Vec<&'static str> {
    CONNECTORS.iter().map(|connector| connector.key).collect()
}

/// Runs a future on a single-threaded tokio runtime.
pub fn block_on<F: Future>(future: F) -> Result<F::Output, Error> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(Error::other)?;
    Ok(runtime.block_on(future))
}

/// rustls client configuration with the Mozilla root certificates, or one
/// that accepts any certificate (`rejectUnauthorized: false`,
/// `trustServerCertificate=true`, `sslmode=require`).
pub fn tls_config(verify: bool) -> rustls::ClientConfig {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let builder = rustls::ClientConfig::builder_with_provider(provider.clone())
        .with_safe_default_protocol_versions()
        .expect("default TLS protocol versions");
    if verify {
        let roots = rustls::RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
        };
        builder.with_root_certificates(roots).with_no_client_auth()
    } else {
        builder
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(AcceptAnyCertificate(provider)))
            .with_no_client_auth()
    }
}

#[derive(Debug)]
struct AcceptAnyCertificate(Arc<rustls::crypto::CryptoProvider>);

impl rustls::client::danger::ServerCertVerifier for AcceptAnyCertificate {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

/// `parseInt(value, 10) || DEFAULT_CONNECTION_TIMEOUT_MS` for the
/// `connectionTimeoutMs` connection string parameter.
pub fn timeout_ms(value: Option<&str>) -> u64 {
    let Some(value) = value else {
        return DEFAULT_CONNECTION_TIMEOUT_MS;
    };
    let value = value.trim_start();
    let digits: String = value
        .char_indices()
        .take_while(|(i, c)| c.is_ascii_digit() || (*i == 0 && (*c == '-' || *c == '+')))
        .map(|(_, c)| c)
        .collect();
    match digits.parse::<i64>() {
        Ok(ms) if ms > 0 => ms as u64,
        _ => DEFAULT_CONNECTION_TIMEOUT_MS,
    }
}

/// Prompts for a connection timeout, as all network connectors do.
pub fn prompt_timeout() -> Result<String, String> {
    let timeout = crate::prompt::input(&format!(
        "Connection timeout in ms (default: {DEFAULT_CONNECTION_TIMEOUT_MS})"
    ))?;
    Ok(if timeout.is_empty() {
        DEFAULT_CONNECTION_TIMEOUT_MS.to_string()
    } else {
        timeout
    })
}

/// Prompts for a value with a default used when the answer is empty
/// (`await Input.prompt(message) || default`).
pub fn prompt_or(message: &str, default: &str) -> Result<String, String> {
    let value = crate::prompt::input(message)?;
    Ok(if value.is_empty() {
        default.to_string()
    } else {
        value
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_timeouts_like_parse_int() {
        assert_eq!(timeout_ms(None), DEFAULT_CONNECTION_TIMEOUT_MS);
        assert_eq!(timeout_ms(Some("5000")), 5000);
        assert_eq!(timeout_ms(Some("5000ms")), 5000);
        assert_eq!(timeout_ms(Some("abc")), DEFAULT_CONNECTION_TIMEOUT_MS);
        assert_eq!(timeout_ms(Some("0")), DEFAULT_CONNECTION_TIMEOUT_MS);
    }
}
