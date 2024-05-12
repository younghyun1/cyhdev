use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
};

use crate::server_init::server_state_model::ServerState;

pub const QUOTE_NUMBER: usize = 13;

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
                    font-size: 0.8em;
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

const QUOTATION_LIST: [&'static str; QUOTE_NUMBER] = [
    "We are made for action, and activity is the sovereign remedy for all physical ills. -Friedrich II of Prussia",
    "They are not capable of understanding a brilliant language but we want to use them to build good software. So, the language that we give them has to be easy for them to understand and easy to adopt. -Rob Pike, on the Go Programming Language",
    "You copied that function without understanding why it does what it does, and as a result your code IS GARBAGE. AGAIN. -Linus Torvalds",
    "Hell is other people. -Jean-Paul Sartre",
    "We can only see a short distance ahead, but we can see plenty there that needs to be done. -Alan Turing",
    "There are only two hard things in Computer Science: cache invalidation and naming things. -Phil Karlton",
    "The best thing about a boolean is even if you are wrong, you are only off by a bit. -Anonymous",
    "Premature optimization is the root of all evil. -Donald Knuth",
    "I believe myself to possess a most singular combination of qualities exactly fitted to make me pre-eminently a discoverer of the hidden realities of nature. -Ada Lovelace",
    "Talk is cheap. Show me the code. -Linus Torvalds",
    "I was told I'd never make it as a programmer because women are good at arts and crafts...but I knew I could program. -Jean Bartik",
    "The only way to learn a new programming language is by writing programs in it. -Dennis Ritchie",
    "If debugging is the process of removing software bugs, then programming must be the process of putting them in. -Edsger Dijkstra",
];
