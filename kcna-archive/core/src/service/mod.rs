//! Read-only archive service: request validation, search orchestration, and response models.

pub mod archive;
#[cfg(test)]
mod archive_tests;
pub mod document;
pub mod export;
pub mod models;
pub mod params;
pub mod trends;
