//! Built-in PostgreSQL functions used by account identity predicates.

use diesel::sql_types::Text;

diesel::define_sql_function! {
    /// PostgreSQL `lower(text)`. Identity predicates compare `lower(column) = lower($1)` so
    /// they match the `users_user_email_lower_unique` and `users_user_name_lower_unique`
    /// expression indexes exactly.
    fn lower(value: Text) -> Text;
}
