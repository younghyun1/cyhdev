//! Serving side of the KCNA research archive.
//!
//! The collector (`kcna-archive`, in `../cli`) keeps PostgreSQL as its store
//! of record and writes a snapshot: one Tantivy index that holds everything a
//! reader needs, plus a small manifest. This crate owns that index layout, the
//! analyzer shared by indexing, querying, and highlighting, and the read-only
//! service that answers every request from the snapshot alone. It has no
//! database, HTTP, or async runtime dependency, so the website can hold the
//! whole archive in memory and call it from its blocking pool.

pub mod error;
pub mod search;
pub mod search_text;
pub mod service;
pub mod snapshot;
pub mod types;
