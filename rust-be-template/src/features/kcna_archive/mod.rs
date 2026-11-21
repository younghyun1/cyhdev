//! The KCNA research archive, read from an in-memory snapshot.
//!
//! The archive is a prebuilt snapshot of public Korean Central News Agency
//! articles, written by the separate collector and read here through the
//! `kcna-archive-core` crate. The service loads the snapshot directory into
//! memory in the background at startup and answers every read from it, without
//! PostgreSQL. The one thing stored in PostgreSQL is the reader list:
//! superusers can always read the archive, and a superuser lists the other
//! accounts that may. Responses are never cached.

pub mod api;
pub mod composition;
pub mod domain;
pub mod error;
pub mod repository;
pub mod service;
