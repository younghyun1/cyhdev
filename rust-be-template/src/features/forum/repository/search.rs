//! Typed PostgreSQL full-text search expressions.

use diesel::query_builder::QueryId;

use crate::schema::sql_types::{ForumSearchQuery, ForumSearchVector};

impl QueryId for ForumSearchVector {
    type QueryId = ForumSearchVector;
    const HAS_STATIC_QUERY_ID: bool = true;
}

impl QueryId for ForumSearchQuery {
    type QueryId = ForumSearchQuery;
    const HAS_STATIC_QUERY_ID: bool = true;
}

diesel::define_sql_function! {
    fn forum_websearch_to_tsquery(query_text: diesel::sql_types::Text) -> ForumSearchQuery;
}

diesel::infix_operator!(ForumSearchMatches, " @@ ", diesel::sql_types::Bool, backend: diesel::pg::Pg);
