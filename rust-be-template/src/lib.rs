#![recursion_limit = "256"]
// Diesel derives emit this lint at field declaration spans; there is no
// handwritten struct initializer to shorten at those locations.
#![allow(clippy::redundant_field_names)]

pub mod build_info;
pub mod docs;
mod docs_registry;
pub mod dto;
pub mod errors;
pub mod features;
pub mod init;
pub mod jobs;
pub mod openapi_codegen;
pub mod openapi_envelope;
pub mod persistence;
pub mod routers;
pub mod schema;
pub mod util;

pub const DOMAIN_NAME: &str = "cyhdev.com";
pub const LOGS_DIR: &str = "./logs/";
