//! Backend integration tests, linked as a single test binary.
//!
//! Cargo links every top-level file in `tests/` as its own executable, and each
//! one carries the whole backend and its dependency graph. One target keeps that
//! to one link per build. PostgreSQL cases under `postgres` are `#[ignore]`d and
//! run through `cargo xtask db-integration` and `cargo xtask migration-rollback`;
//! `tests/README.md` describes them.

mod markdown_rendering;
mod media_cleanup_compensation;
mod postgres;
mod support;
