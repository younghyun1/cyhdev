use std::{sync::Arc, time::Instant};

use anyhow::{anyhow, Result};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use serde_derive::Serialize;
use sysinfo::System;

use crate::{
    server_init::server_state_model::ServerState,
    utils::{
        misc_utils::bool_to_emoji::bte,
        time_utils::format_datetime_difference::format_dt_difference,
    },
    APP_NAME_VERSION,
};

// 요청에 대한 답변 JSON 구조를 여기서 정의한다. Debug, Serialize를 derive 해야 서버 로그, 클라이언트 답변으로 자동 변환 가능.
// JSON structure for the request reply is defined here. Derive Debug and Serialized in order to auto-convert.
#[derive(Debug, Serialize)]
pub struct HealthCheckReplyJson {
    message: Vec<String>,
    status: bool,
    database_connection: bool,
    database_latency: String,
    cpu_use: f32,
    ram_use: f32,
    timestamp: DateTime<Utc>,
}

// GET '/healthcheck/systemcheck'
// Healthcheck 요청을 처리하는 함수.
// Processes healthcheck request.
pub async fn systemcheck_handler(
    State(state): State<Arc<ServerState>>,
    //서버 state를 받아온다. Gets server state.
) -> impl IntoResponse {
    // DB 레이턴시 측정을 위한 시간 측정 시작.
    // Record current time for DB latency check.
    let start = Instant::now();

    // 여기 DB랑 연결해서 연결 되는지 확인해보고 레이턴시 측정한다.
    // Connect to DB here, check if connection is possible, also measure latency.
    let database_connection: Result<String> =
        check_database_connection(Arc::clone(&state.pool)).await;
    let duration_db_ping: std::time::Duration = start.elapsed();

    // DB 연결 말고도 확인할만한게 있을 수 있으니 Vec<bool>로 성공 조건을 넣어줄 것.
    // Potentially many other things to check apart from DB connection. Push them all into the Vec<bool>.
    let mut checks_vec: Vec<bool> = Vec::new();
    checks_vec.push(database_connection.is_ok());

    // 여기서 sys_info 라이브러리를 사용하여 CPU, RAM 상태를 기록.
    // Use the sys_info library here to record the CPU and RAM state.
    let mut system: System = System::new_all();
    let cpu_usage: f32 = system.global_cpu_usage();
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    // Refresh CPUs again.
    system.refresh_all();
    let total_memory: u64 = system.total_memory();
    let used_memory: u64 = system.used_memory();
    let ram_usage: f32 = ((used_memory as f64 / total_memory as f64) * 100.0) as f32;

    // 시간대 정보 포함 시간 정보 생성.
    // Generate timestamp with timezone info here.
    let current_time: DateTime<Utc> = Utc::now();

    // 모든 조건이 다 true인지 확인. .all 람다 method는 Iterable 특성을 가지고 있는 객체에 사용 가능한 method이다.
    // Check if all checks are good. Use the .all lambda method on any variable with the Iterator trait to check for all.
    let all_good: bool = checks_vec.iter().all(|&x| x);
    let mut message: Vec<String> = Vec::new();

    // 통과 조건에 따른 message 포맷팅.
    // Message formatting depending on whether all checks were passed.
    if all_good {
        message.push(format!("{}", APP_NAME_VERSION));
        message.push(format!(
            "Database connection: {}; message: {}",
            bte(database_connection.is_ok()),
            match database_connection.as_ref() {
                Ok(str) => str,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Could not parse database connection message: {:?}", e),
                    )
                        .into_response();
                }
            },
        ));
        message.push(format!(
            "Quotes delivered: {}",
            state.quote_shuffle_bag.read().await.count,
        ));
        message.push(format!(
            "Uptime: {}",
            format_dt_difference(state.server_start_time, current_time)
        ));
        message.push("Good to go!".to_string());
    } else {
        message.push(format!(
            "Database connection: {}",
            bte(database_connection.is_ok())
        ));
        message.push(format!(
            "Quotes delivered: {}",
            state.quote_shuffle_bag.read().await.count,
        ));
        message.push(format!(
            "Uptime: {}",
            format_dt_difference(state.server_start_time, current_time)
        ));
        message.push("Server down!".to_string());
    }

    // 여기서 답변 만듬.
    // Make reply here.
    let healthcheck_reply: HealthCheckReplyJson = HealthCheckReplyJson {
        message: message,
        status: all_good,
        database_connection: database_connection.is_ok(),
        database_latency: format!("{:?}", duration_db_ping),
        cpu_use: cpu_usage,
        ram_use: ram_usage,
        timestamp: current_time,
    };

    // 성공 조건에 따라 로깅하고 status code 붙여서 답변함. 나중에 에러 메세지 다양하게 해야 될수도.
    // Log and reply with the appropriate status code. May need to diversify error type more.
    if all_good {
        (StatusCode::OK, Json(healthcheck_reply)).into_response()
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(healthcheck_reply)).into_response()
    }
}

// DB URL 받아서 연결되는지 간단히 확인하는 함수.
// Get DB URL and check for connection.
pub async fn check_database_connection(pool: Arc<Pool>) -> Result<String> {
    let client = match pool.get().await {
        Ok(client) => client,
        Err(e) => {
            return Err(anyhow!(
                "Failed to get client from pool during systemcheck: {:?}",
                e
            ));
        }
    };

    let rows = match client.query("SELECT $1::TEXT", &[&"hello world"]).await {
        Ok(rows) => rows,
        Err(e) => {
            return Err(anyhow!("Failed to SELECT hello world: {:?}", e));
        }
    };

    return Ok(rows[0].get(0));
}
