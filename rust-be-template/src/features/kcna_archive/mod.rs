//! Administrator-only access to the KCNA research archive.
//!
//! The archive is a prebuilt snapshot of public Korean Central News Agency
//! articles, written by the separate collector and read here through the
//! `kcna-archive-core` crate. The snapshot is the data source, so this slice
//! has no repository layer and no PostgreSQL access: the service loads the
//! snapshot directory into memory in the background at startup and answers every
//! request from it. The corpus is not redistributable, so every route sits
//! behind the superuser gate and responses are never cached.

pub mod api;
pub mod error;
pub mod service;
