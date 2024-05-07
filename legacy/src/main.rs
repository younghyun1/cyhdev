use anyhow::{anyhow, Result};
use server_init::server_init::server_initializer;

mod db_io {
    pub mod schema;
    pub mod schema_rust;
}
mod handlers {
    pub mod api {
        pub mod submit_message;
    }
    pub mod health_check;
    pub mod health_check_aws;
}
mod server_init {
    pub mod server_init;
    pub mod state_init;
}
mod utils {
    pub mod bool_to_emoji;
}

#[tokio::main(flavor = "multi_thread")]
pub async fn main() -> Result<()> {
    match dotenvy::dotenv() {
        Ok(path_buf) => {
            println!("Env. variables at {:?} loaded!", path_buf.to_str());
        }
        Err(e) => {
            return Err(anyhoww!(e).context("dotenvy could not load .env file! please check if an environment variable file exists."));
        }
    }

    match server_initializer().await {
        Ok(server_initializer_result) => {
            println!(
                "Server successfully terminated:\n{}",
                server_initializer_result
            );
            Ok(())
        }
        Err(e) => Err(anyhow!(e).context("Server did not successfully initialize.")),
    }
}
