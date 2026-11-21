//! PostgreSQL corpus modules.

pub mod domain;
#[cfg(test)]
mod integration_tests;
pub mod migrate;
pub mod models;
pub mod pool;
pub mod schema;
pub mod store;
pub mod store_links;
mod store_retrievals;
#[cfg(test)]
mod test_reads;
pub mod types;
