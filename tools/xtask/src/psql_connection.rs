//! Splits a libpq connection URI so its password travels in the child
//! environment instead of the process argument list, which any local user can
//! read through `/proc/<pid>/cmdline` or `ps`.

use std::{ffi::OsString, os::unix::ffi::OsStringExt};

use crate::{TaskError, TaskResult};

/// A password-free libpq URI plus the decoded password for `PGPASSWORD`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PsqlConnection {
    pub(crate) url: String,
    pub(crate) password: Option<OsString>,
}

impl PsqlConnection {
    /// Parses `postgres://[user[:password]@][hosts][/database][?parameters]`
    /// the way libpq does: the user information ends at the first `@` that
    /// precedes any `/`, the password runs from the first `:` to that `@`, and
    /// both it and a `password=` query parameter are percent-decoded. Errors
    /// never echo the URI, since it may contain the password.
    pub(crate) fn parse(url: &str) -> TaskResult<Self> {
        let (scheme, rest) = match url.strip_prefix("postgresql://") {
            Some(rest) => ("postgresql://", rest),
            None => match url.strip_prefix("postgres://") {
                Some(rest) => ("postgres://", rest),
                None => {
                    return Err(TaskError(
                        "TEST_DATABASE_URL must use postgres:// or postgresql://".to_owned(),
                    ));
                }
            },
        };

        let (userinfo, location) = match rest.find(['@', '/']) {
            Some(index) => {
                let (before, after) = rest.split_at(index);
                match after.strip_prefix('@') {
                    Some(location) => (Some(before), location),
                    None => (None, rest),
                }
            }
            None => (None, rest),
        };
        let (user, userinfo_password) = match userinfo {
            Some(info) => match info.split_once(':') {
                Some((user, password)) => (user, Some(password)),
                None => (info, None),
            },
            None => ("", None),
        };

        let (base, query) = match location.split_once('?') {
            Some((base, query)) => (base, Some(query)),
            None => (location, None),
        };
        let mut query_password = None;
        let mut kept_parameters = Vec::new();
        for parameter in query.into_iter().flat_map(|query| query.split('&')) {
            // libpq decodes parameter names too, so `pass%77ord` also counts.
            match parameter.split_once('=') {
                Some((key, value)) if percent_decode(key)? == b"password" => {
                    if query_password.replace(value).is_some() {
                        return Err(repeated_password());
                    }
                }
                _ => kept_parameters.push(parameter),
            }
        }

        // libpq ignores an empty password, so an empty value selects none.
        let password = match (
            userinfo_password.filter(|value| !value.is_empty()),
            query_password.filter(|value| !value.is_empty()),
        ) {
            (Some(_), Some(_)) => return Err(repeated_password()),
            (Some(encoded), None) | (None, Some(encoded)) => {
                Some(OsString::from_vec(percent_decode(encoded)?))
            }
            (None, None) => None,
        };

        let mut stripped = String::with_capacity(url.len());
        stripped.push_str(scheme);
        if !user.is_empty() {
            stripped.push_str(user);
            stripped.push('@');
        }
        stripped.push_str(base);
        if !kept_parameters.is_empty() {
            stripped.push('?');
            stripped.push_str(&kept_parameters.join("&"));
        }
        Ok(Self {
            url: stripped,
            password,
        })
    }
}

fn repeated_password() -> TaskError {
    TaskError("TEST_DATABASE_URL must specify its password at most once".to_owned())
}

/// Decodes `%XX` escapes and rejects malformed escapes and `%00`, as libpq does.
fn percent_decode(value: &str) -> TaskResult<Vec<u8>> {
    let invalid = || TaskError("TEST_DATABASE_URL contains an invalid percent escape".to_owned());
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        if byte != b'%' {
            decoded.push(byte);
            index += 1;
            continue;
        }
        let digit = |offset: usize| {
            bytes
                .get(index + offset)
                .and_then(|digit| char::from(*digit).to_digit(16))
        };
        let (Some(high), Some(low)) = (digit(1), digit(2)) else {
            return Err(invalid());
        };
        let byte = u8::try_from(high * 16 + low).map_err(|_| invalid())?;
        if byte == 0 {
            return Err(invalid());
        }
        decoded.push(byte);
        index += 3;
    }
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::PsqlConnection;

    fn split(url: &str) -> crate::TaskResult<(String, Option<OsString>)> {
        PsqlConnection::parse(url).map(|connection| (connection.url, connection.password))
    }

    #[test]
    fn userinfo_password_moves_out_of_the_uri() -> crate::TaskResult<()> {
        assert_eq!(
            split("postgres://tester:p%40ss:w%2Frd@localhost:5432/cyhdev_test_maintenance")?,
            (
                "postgres://tester@localhost:5432/cyhdev_test_maintenance".to_owned(),
                Some(OsString::from("p@ss:w/rd"))
            )
        );
        assert_eq!(
            split("postgresql://tester:secret@[::1]:5432/db?sslmode=disable")?,
            (
                "postgresql://tester@[::1]:5432/db?sslmode=disable".to_owned(),
                Some(OsString::from("secret"))
            )
        );
        assert_eq!(
            split("postgres://:secret@/db?host=/run/postgresql")?,
            (
                "postgres:///db?host=/run/postgresql".to_owned(),
                Some(OsString::from("secret"))
            )
        );
        Ok(())
    }

    #[test]
    fn query_password_moves_out_and_other_parameters_stay() -> crate::TaskResult<()> {
        assert_eq!(
            split(
                "postgres://tester@localhost/db?sslmode=disable&password=se%2Fcret&application_name=xtask"
            )?,
            (
                "postgres://tester@localhost/db?sslmode=disable&application_name=xtask".to_owned(),
                Some(OsString::from("se/cret"))
            )
        );
        assert_eq!(
            split("postgres://localhost/db?password=secret")?,
            (
                "postgres://localhost/db".to_owned(),
                Some(OsString::from("secret"))
            )
        );
        Ok(())
    }

    #[test]
    fn password_free_uris_are_unchanged() -> crate::TaskResult<()> {
        for url in [
            "postgres://tester@localhost/db",
            "postgresql://localhost:5432/db?sslmode=disable",
            "postgres://localhost/db@suffix",
            "postgres://tester:@localhost/db",
        ] {
            let (stripped, password) = split(url)?;
            assert_eq!(password, None, "{url}");
            assert_eq!(stripped, url.replacen("tester:@", "tester@", 1));
        }
        Ok(())
    }

    #[test]
    fn ambiguous_or_malformed_passwords_fail_without_echoing_them() -> crate::TaskResult<()> {
        for url in [
            "postgres://tester:first@localhost/db?password=second",
            "postgres://tester@localhost/db?password=first&password=second",
            "postgres://tester:bad%zzsecret@localhost/db",
            "postgres://tester:trunc%4@localhost/db",
            "postgres://tester:nul%00secret@localhost/db",
            "postgres://tester@localhost/db?pass%77ord=first&password=second",
            "mysql://tester:secret@localhost/db",
        ] {
            match PsqlConnection::parse(url) {
                Ok(connection) => {
                    return Err(crate::TaskError(format!(
                        "accepted {url} as {}",
                        connection.url
                    )));
                }
                Err(error) => {
                    let message = error.to_string();
                    assert!(!message.contains("secret"), "{message}");
                    assert!(!message.contains("first"), "{message}");
                }
            }
        }
        Ok(())
    }
}
