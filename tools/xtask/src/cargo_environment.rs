//! Keeps xtask's own package description out of the commands it starts.
//!
//! `cargo xtask` runs through `cargo run`, which sets `CARGO_MANIFEST_DIR` and
//! `CARGO_PKG_*` to xtask's values, and every command xtask starts inherits
//! them. Build scripts such as ring's watch those names through
//! `rerun-if-env-changed`, and Cargo compares the watched values with its own
//! environment. Alternating `cargo xtask unit` with a direct `cargo test` therefore
//! reran ring's build script and rebuilt the roughly 30 crates above it.

use std::{env, ffi::OsStr};

/// Whether `name` is a variable `cargo run` sets to describe the package it
/// runs. User configuration such as `CARGO_HOME` or `CARGO_TARGET_DIR` is not.
fn describes_running_package(name: &OsStr) -> bool {
    match name.to_str() {
        Some(name) => {
            name.starts_with("CARGO_PKG_")
                || matches!(
                    name,
                    "CARGO_MANIFEST_DIR"
                        | "CARGO_MANIFEST_PATH"
                        | "CARGO_CRATE_NAME"
                        | "CARGO_BIN_NAME"
                        | "CARGO_PRIMARY_PACKAGE"
                )
        }
        None => false,
    }
}

/// Removes those variables so child commands see the caller's environment.
pub(crate) fn clear_running_package() {
    let names: Vec<_> = env::vars_os()
        .map(|(name, _)| name)
        .filter(|name| describes_running_package(name))
        .collect();
    for name in names {
        // Called first in `main`, before xtask starts any thread, so nothing
        // else reads the environment while it changes.
        unsafe { env::remove_var(name) };
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::describes_running_package;

    #[test]
    fn only_the_running_package_description_is_removed() {
        for name in [
            "CARGO_MANIFEST_DIR",
            "CARGO_MANIFEST_PATH",
            "CARGO_PKG_NAME",
            "CARGO_PKG_VERSION_MAJOR",
            "CARGO_CRATE_NAME",
            "CARGO_BIN_NAME",
            "CARGO_PRIMARY_PACKAGE",
        ] {
            assert!(describes_running_package(OsStr::new(name)), "{name}");
        }
        for name in [
            "CARGO",
            "CARGO_HOME",
            "CARGO_TARGET_DIR",
            "CARGO_INCREMENTAL",
            "CARGO_BUILD_JOBS",
            "RUSTUP_TOOLCHAIN",
            "PATH",
        ] {
            assert!(!describes_running_package(OsStr::new(name)), "{name}");
        }
    }
}
