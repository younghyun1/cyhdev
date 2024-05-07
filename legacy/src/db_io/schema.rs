// @generated automatically by Diesel CLI.

pub mod v1 {
    diesel::table! {
        v1.messages (message_id) {
            message_id -> Uuid,
            submitted_at -> Timestamp,
            title -> Varchar,
            body -> Varchar,
            name -> Varchar,
        }
    }
}
