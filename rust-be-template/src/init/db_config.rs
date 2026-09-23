//! Database connection settings and their URL form.
//!
//! Components are stored decoded and percent-encoded only when the URL is built, so
//! a password or database name containing `@`, `:`, `/`, `?`, `#`, `%`, or spaces
//! reaches libpq (migrations) and tokio-postgres (pool) unchanged. Both parsers
//! percent-decode every URL component, including query values.

use std::net::Ipv6Addr;

use anyhow::anyhow;

enum DbType {
    Postgres,
    MySql,
    Sqlite,
    Oracle,
    MsSql,
}

pub struct DbConfig {
    db_type: DbType,
    /// Host name, IP literal, or absolute Unix-socket directory.
    db_host: String,
    db_port: Option<u16>,
    db_username: String,
    db_password: String,
    db_name: String,
    /// Other decoded `DB_URL` query parameters, such as `sslmode`, preserved in order.
    db_params: Vec<(String, String)>,
}

impl DbConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let is_socket_path = std::env::var("DB_HOST")
            .ok()
            .is_some_and(|host| host.starts_with('/'));

        if !is_socket_path && let Ok(db_url) = std::env::var("DB_URL") {
            return Self::from_url(&db_url);
        }

        let db_host = std::env::var("DB_HOST")
            .map_err(|_| anyhow!("Environment variable DB_HOST not found"))?;

        let db_port = if db_host.starts_with('/') {
            None
        } else {
            Some(
                std::env::var("DB_PORT")
                    .map_err(|_| anyhow!("Environment variable DB_PORT not found"))?
                    .parse::<u16>()?,
            )
        };

        Ok(DbConfig {
            db_type: DbType::Postgres,
            db_host,
            db_port,
            db_username: std::env::var("DB_USERNAME")
                .map_err(|_| anyhow!("Environment variable DB_USERNAME not found"))?,
            db_password: std::env::var("DB_PASSWORD")
                .map_err(|_| anyhow!("Environment variable DB_PASSWORD not found"))?,
            db_name: std::env::var("DB_NAME")
                .map_err(|_| anyhow!("Environment variable DB_NAME not found"))?,
            db_params: Vec::new(),
        })
    }

    /// Parses `scheme://user:password@host:port/database?params`, decoding every component.
    ///
    /// The last `@` separates credentials so an unencoded `@` in a legacy password still
    /// parses. A `host` query parameter overrides the authority host, which is how
    /// libpq expresses a Unix-socket directory (`postgres://user@/db?host=/run/postgresql`).
    pub fn from_url(url: &str) -> anyhow::Result<Self> {
        let (scheme, rest) = url
            .split_once("://")
            .ok_or_else(|| anyhow!("Invalid URL format"))?;

        let db_type = match scheme.trim().to_lowercase().as_ref() {
            "postgres" | "psql" | "postgresql" | "pg" => DbType::Postgres,
            "mysql" | "mariadb" | "maria" => DbType::MySql,
            "sqlite" | "sqlite3" => DbType::Sqlite,
            "oracle" | "ora" | "orcl" => DbType::Oracle,
            "mssql" | "microsoftsql" | "sqlserver" => DbType::MsSql,
            _ => {
                return Err(anyhow!(
                    "Unsupported DB; only postgreSQL is supported for now."
                ));
            }
        };

        let (location, query) = match rest.split_once('?') {
            Some((location, query)) => (location, Some(query)),
            None => (rest, None),
        };
        let (credentials, host_and_path) = location
            .rsplit_once('@')
            .ok_or_else(|| anyhow!("Missing credentials"))?;
        let (db_username, db_password) = match credentials.split_once(':') {
            Some((user, password)) => (percent_decode(user)?, percent_decode(password)?),
            None => (percent_decode(credentials)?, String::new()),
        };
        let (host_and_port, db_name) = match host_and_path.split_once('/') {
            Some((host_and_port, db_name)) => (host_and_port, percent_decode(db_name)?),
            None => (host_and_path, String::new()),
        };
        let (mut db_host, explicit_port) = split_host_and_port(host_and_port)?;

        let mut db_params = Vec::new();
        for pair in query.into_iter().flat_map(|query| query.split('&')) {
            if pair.is_empty() {
                continue;
            }
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            let (key, value) = (percent_decode(key)?, percent_decode(value)?);
            if key == "host" {
                db_host = value;
            } else {
                db_params.push((key, value));
            }
        }

        let db_port = if db_host.starts_with('/') {
            None
        } else {
            match explicit_port {
                Some(port) => Some(port),
                None => Some(match db_type {
                    DbType::Postgres => 5432,
                    DbType::MySql => 3306,
                    DbType::Sqlite => 0,
                    DbType::Oracle => 1521,
                    DbType::MsSql => 1433,
                }),
            }
        };

        Ok(DbConfig {
            db_type,
            db_host,
            db_port,
            db_username,
            db_password,
            db_name,
            db_params,
        })
    }

    /// Builds the connection URL with every component percent-encoded.
    pub fn to_url(&self) -> anyhow::Result<String> {
        let scheme = match self.db_type {
            DbType::Postgres => "postgres",
            DbType::MySql => "mysql",
            DbType::Sqlite => "sqlite",
            DbType::Oracle => "oracle",
            DbType::MsSql => "mssql",
        };
        let user = percent_encode(&self.db_username);
        let password = percent_encode(&self.db_password);
        let db_name = percent_encode(&self.db_name);
        let mut params = Vec::with_capacity(self.db_params.len() + 1);

        // A socket directory is a path, so it travels as the `host` query parameter.
        let authority = if self.db_host.starts_with('/') {
            params.push(format!("host={}", percent_encode(&self.db_host)));
            String::new()
        } else {
            let host = match self.db_host.parse::<Ipv6Addr>() {
                Ok(address) => format!("[{address}]"),
                Err(_) => percent_encode(&self.db_host),
            };
            match self.db_port {
                Some(port) => format!("{host}:{port}"),
                None => host,
            }
        };
        params.extend(
            self.db_params
                .iter()
                .map(|(key, value)| format!("{}={}", percent_encode(key), percent_encode(value))),
        );

        let mut url = format!("{scheme}://{user}:{password}@{authority}/{db_name}");
        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }
        Ok(url)
    }
}

/// Splits `host:port` or `[ipv6]:port`; the port is optional.
fn split_host_and_port(value: &str) -> anyhow::Result<(String, Option<u16>)> {
    if let Some(bracketed) = value.strip_prefix('[') {
        let (address, remainder) = bracketed
            .split_once(']')
            .ok_or_else(|| anyhow!("Unterminated IPv6 host"))?;
        let port = match remainder.strip_prefix(':') {
            Some(port) => Some(port.parse::<u16>()?),
            None if remainder.is_empty() => None,
            None => return Err(anyhow!("Invalid characters after IPv6 host")),
        };
        return Ok((percent_decode(address)?, port));
    }
    match value.split_once(':') {
        Some((host, port)) => Ok((percent_decode(host)?, Some(port.parse::<u16>()?))),
        None => Ok((percent_decode(value)?, None)),
    }
}

/// Encodes everything except RFC 3986 unreserved characters, which is valid in the
/// user-information, host, path, and query components alike.
pub(crate) fn percent_encode(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push(char::from(HEX[usize::from(byte >> 4)]));
            encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
    }
    encoded
}

/// Decodes `%XX` escapes; a `%` that does not start a valid escape stays literal, as
/// in the parsers that later read the URL.
fn percent_decode(value: &str) -> anyhow::Result<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        let escaped = match (byte, bytes.get(index + 1), bytes.get(index + 2)) {
            (b'%', Some(&high), Some(&low)) => match (hex_value(high), hex_value(low)) {
                (Some(high), Some(low)) => Some((high << 4) | low),
                _ => None,
            },
            _ => None,
        };
        match escaped {
            Some(value) => {
                decoded.push(value);
                index += 3;
            }
            None => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(decoded).map_err(|_| anyhow!("Database URL component is not UTF-8"))
}

fn hex_value(digit: u8) -> Option<u8> {
    match digit {
        b'0'..=b'9' => Some(digit - b'0'),
        b'a'..=b'f' => Some(digit - b'a' + 10),
        b'A'..=b'F' => Some(digit - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
#[path = "db_config_tests.rs"]
mod tests;
