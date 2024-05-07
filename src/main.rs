use std::net::{IpAddr, Ipv4Addr};

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use server_init::server_init::server_initializer;
use tracing::info;

mod server_init {
    pub mod server_init;
    pub mod server_state_model;
    pub mod state_init;
}
mod utils {
    pub mod bool_to_emoji;
}

pub const APP_NAME_VERSION: &'static str = "cyhdev-0.0.1";
pub const DB_ADDR: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);
pub const DB_PORT: u16 = 5432;
pub const DB_USERNAME: &'static str = "postgres";

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let server_start_time: DateTime<Utc> = Utc::now();
    let server_start: tokio::time::Instant = tokio::time::Instant::now();

    // All crates that use the 'tracing' or 'log' libraries publishes to this thing.
    // tracing 또는 log 크레이트 형태를 사용하는 로그를 남기는 모든 크레이트가 여기 정의된 형식을 통해서 로그를 표출함.
    tracing_subscriber::fmt()
        // .with_max_level(tracing::Level::DEBUG) // for testing
        .with_max_level(tracing::Level::INFO) // for release
        .with_ansi(false) // disables colored output
        .init();

    // No terminal echo when inputting confidential information at runtime.
    // 런타임에서 민감한 정보를 stdin을 통해서 입력할때 에코 제거.
    let pw: String = match rpassword::prompt_password("DB_PASSWORD: ") {
        Ok(pw) => pw,
        Err(_) => {
            return Err(anyhow!("I/O error in reading DB password."));
        }
    };

    // 유닛 테스트를 위하여 서버 시작 부분 논리는 분리해놓음
    // Server initialization logic separated for potential future unit testing.
    match server_initializer(server_start, server_start_time, pw).await {
        Ok(_) => {
            info!("Server successfully terminated.",);
            return Ok(());
        }
        Err(e) => {
            return Err(anyhow!("Server exiting with error: {:?}", e));
        }
    }
}
