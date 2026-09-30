//! Disposable, synthetic dependencies for native optimization campaigns.

mod campaign;
mod database;
mod environment;
mod files;
mod management;
mod minecraft;
mod provider;
mod retention;
mod seed;
mod smtp;
mod web;

use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_| anyhow::anyhow!("TLS provider was already initialized"))?;
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    anyhow::ensure!(
        arguments.len() == 2,
        "usage: optimization-fixtures <prepare|reset|serve|campaign> RUNTIME"
    );
    let runtime = PathBuf::from(&arguments[1]);
    files::validate_runtime(&runtime)?;
    match arguments[0].as_str() {
        "prepare" => {
            files::prepare(&runtime)?;
            database::prepare(&runtime).await?;
        }
        "reset" => database::reset(&runtime).await?,
        "campaign" => campaign::write(&runtime)?,
        "serve" => {
            tokio::select! {
                result = async { tokio::try_join!(web::serve(runtime.clone()), smtp::serve(runtime.clone()), minecraft::serve(runtime.clone())) } => { result?; },
                result = shutdown() => { result?; },
            }
            for name in ["control.sock", "world.sock"] {
                let path = runtime.join(name);
                if path.exists() {
                    std::fs::remove_file(path)?;
                }
            }
        }
        _ => anyhow::bail!("unknown fixture command"),
    }
    Ok(())
}

async fn shutdown() -> anyhow::Result<()> {
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    tokio::select! { result = tokio::signal::ctrl_c() => { result?; }, _ = terminate.recv() => {} }
    Ok(())
}
