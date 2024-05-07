use std::net::{IpAddr, Ipv4Addr};

use anyhow::{anyhow, Result};

pub const APP_NAME_VERSION: &'static str = "cyhdev-0.0.1";
pub const DB_ADDR: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);
pub const DB_PORT: u16 = 5432;
pub const DB_USERNAME: &'static str = "postgres";

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let server_start_time: tokio::time::Instant = tokio::time::Instant::now();

    // all crates that use the 'tracing' or 'log' libraries publishes to this thing.
    // tracing 또는 log 크레이트 형태를 사용하는 로그를 남기는 모든 크레이트가 여기 정의된 형식을 통해서 로그를 표출함.
    tracing_subscriber::fmt()
        // .with_max_level(tracing::Level::DEBUG) // for testing
        .with_max_level(tracing::Level::INFO) // for release
        .with_ansi(false) // disables colored output
        .init();

    let pw: String = match rpassword::prompt_password("DB_PASSWORD: ") {
        Ok(pw) => pw,
        Err(_) => {
            return Err(anyhow!("I/O error in reading DB password."));
        }
    };
    drop(pw);

    Ok(())
}
