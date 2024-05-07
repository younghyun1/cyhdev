use std::net::{IpAddr, Ipv4Addr};

use anyhow::{anyhow, Result};

pub const APP_NAME_VERSION: &'static str = "cyhdev-0.0.1";
pub const DB_ADDR: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);
pub const DB_PORT: u16 = 5432;
pub const DB_USERNAME: &'static str = "postgres";

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let pw: String = match rpassword::prompt_password("DB_PASSWORD: ") {
        Ok(pw) => pw,
        Err(_) => {
            return Err(anyhow!("I/O error in reading DB password."));
        }
    };
    drop(pw);

    Ok(())
}
