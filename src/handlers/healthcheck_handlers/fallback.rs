use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
};

use crate::server_init::server_state_model::ServerState;

use super::quotes::{QUOTATION_LIST, QUOTE_NUMBER};

pub async fn fallback_handler(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let quote_counter_mutex = &mut *state.quote_shuffle_bag.write().await;
    quote_counter_mutex.count += 1;
    if quote_counter_mutex.count % QUOTE_NUMBER == 0 {
        quote_counter_mutex.shuffle_bag();
    }
    let idx_to_query_by =
        quote_counter_mutex.shuffle_bag[(quote_counter_mutex.count % QUOTE_NUMBER) as usize];

    let quote = QUOTATION_LIST[idx_to_query_by as usize];

    let html_page = format!(
        "<!DOCTYPE html>
        <html>
        <head>
            <title>Page Not Found</title>
            <style>
                body {{
                    background-color: black;
                    color: white;
                    font-family: 'Arial', sans-serif;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    height: 100vh;
                    margin: 0;
                }}
                .container {{
                    text-align: center;
                }}
                .quote {{
                    font-size: 1.5em;
                }}
                .footer {{
                    font-size: 1em;
                    margin-top: 20px;
                }}
            </style>
        </head>
        <body>
            <div class='container'>
                <div class='quote'>{}</div>
                <div class='footer'>404 - Not Found</div>
            </div>
        </body>
        </html>",
        quote
    );

    return (StatusCode::NOT_FOUND, Html(html_page));
}
