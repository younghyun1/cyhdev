//! Full-text index over the corpus: layout, writing, querying, and highlighting.

pub mod collectors;
pub mod engine;
#[cfg(test)]
pub(crate) mod engine_tests;
pub mod fields;
pub mod highlight;
pub mod lookup;
#[cfg(test)]
mod lookup_tests;
pub mod query;
pub mod request;
pub mod stored;
pub mod tokenizer;
pub mod writer;
