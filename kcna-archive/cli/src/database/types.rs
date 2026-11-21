//! Strict Rust counterparts for PostgreSQL corpus enums.
//!
//! The values shared with the snapshot and the website live in the core
//! crate, which maps them to PostgreSQL behind its `diesel` feature; what only
//! the importer writes stays here.

use diesel_derive_enum::DbEnum;
pub use kcna_archive_core::types::{Language, PublicationPrecision, Source};

/// How the members of a translation group were matched to each other.
#[derive(Clone, Copy, Debug, DbEnum, Eq, PartialEq)]
#[ExistingTypePath = "crate::database::schema::sql_types::KcnaTranslationLinkMethod"]
pub enum TranslationLinkMethod {
    /// The source publishes one identifier for every language edition.
    SourceIdentifier,
    /// Matched by content; reserved for historical pages without identifiers.
    #[allow(dead_code)]
    Inferred,
}
