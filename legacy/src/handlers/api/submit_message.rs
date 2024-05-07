use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use diesel_async::RunQueryDsl;
use serde_derive::{Deserialize, Serialize};

use crate::{
    db_io::{schema::v1::messages, schema_rust::Message},
    server_init::state_init::ServerState,
};

#[derive(Deserialize)]
pub struct SubmittedMessage {
    pub title: String,
    pub body: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub success: bool,
    pub message: String,
}

// #[debug_handler]
pub async fn submit_handler(
    State(state): State<ServerState>,
    Json(message): Json<SubmittedMessage>,
) -> impl IntoResponse {
    let mut conn = match state.writer_client.get().await {
        Ok(conn) => conn,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                format!("Connection cannot be grabbed from pool: {}", e.to_string()),
            )
                .into_response()
        }
    };

    match diesel::insert_into(messages::table)
        .values(Message {
            title: message.title,
            body: message.body,
            name: message.name,
        })
        .execute(&mut conn)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                format!("Could not insert rows: {}", e.to_string()),
            )
                .into_response()
        }
    };

    return (
        StatusCode::OK,
        Json(MessageResponse {
            success: true,
            message: format!("Message submitted!"),
        }),
    )
        .into_response();
}
