//! Manual loopback experiment isolating TLS/HTTP transport from application and disk latency.

use std::{error::Error, process::Stdio, time::Duration};

use axum::{Router, body::Bytes, routing::get};
use axum_server::{Handle, accept::NoDelayAcceptor, tls_rustls::RustlsConfig};
use tokio::process::Command;

/// Compare identical in-memory responses with only TCP_NODELAY changed. Timing is evidence,
/// not a CI threshold: scheduler load and platform delayed-ACK policy vary independently.
#[tokio::test]
#[ignore = "manual latency experiment; requires openssl and curl with HTTP/2"]
async fn compare_reused_tls_connections() -> Result<(), Box<dyn Error>> {
    let subscriber = tracing_subscriber::fmt()
        .json()
        .flatten_event(true)
        .with_ansi(false)
        .finish();
    let _logging = tracing::subscriber::set_default(subscriber);
    if rustls::crypto::CryptoProvider::get_default().is_none() {
        rustls::crypto::aws_lc_rs::default_provider()
            .install_default()
            .map_err(|_| "cannot install TLS provider")?;
    }
    let directory = tempfile::tempdir()?;
    let certificate = directory.path().join("certificate.pem");
    let key = directory.path().join("key.pem");
    let generated = Command::new("openssl")
        .args([
            "req",
            "-x509",
            "-newkey",
            "rsa:2048",
            "-nodes",
            "-days",
            "1",
            "-subj",
            "/CN=localhost",
            "-addext",
            "subjectAltName=DNS:localhost",
            "-addext",
            "basicConstraints=critical,CA:TRUE",
            "-keyout",
        ])
        .arg(&key)
        .arg("-out")
        .arg(&certificate)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await?;
    if !generated.success() {
        return Err("test certificate generation failed".into());
    }
    let config = RustlsConfig::from_pem_file(&certificate, &key).await?;
    for nodelay in [false, true] {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0))?;
        listener.set_nonblocking(true)?;
        let address = listener.local_addr()?;
        let data = Bytes::from(vec![b'x'; 59_571]);
        let app = Router::new().route(
            "/tile.png",
            get(move || {
                let data = data.clone();
                async move { ([("content-type", "image/png")], data) }
            }),
        );
        let handle = Handle::new();
        let server = axum_server::from_tcp_rustls(listener, config.clone())?.handle(handle.clone());
        let task = tokio::spawn(async move {
            if nodelay {
                server
                    .map(|acceptor| acceptor.acceptor(NoDelayAcceptor::new()))
                    .serve(app.into_make_service())
                    .await
            } else {
                server.serve(app.into_make_service()).await
            }
        });
        if handle.listening().await.is_none() {
            return Err("probe listener failed".into());
        }
        let result = sample(&certificate, address.port(), nodelay).await;
        handle.graceful_shutdown(Some(Duration::from_secs(1)));
        task.await??;
        result?;
    }
    Ok(())
}

/// One curl process reuses its TLS connection; exclude the first transfer from latency summaries.
async fn sample(
    certificate: &std::path::Path,
    port: u16,
    nodelay: bool,
) -> Result<(), Box<dyn Error>> {
    let url = format!("https://localhost:{port}/tile.png");
    for (protocol, expected) in [("--http2", "2"), ("--http1.1", "1.1")] {
        let mut command = Command::new("curl");
        command.args(["--fail", "--silent", "--show-error", "--noproxy", "*", "--max-time", "5", protocol, "--cacert"])
            .arg(certificate).arg("--resolve").arg(format!("localhost:{port}:127.0.0.1"))
            .args(["--write-out", "%{http_version} %{http_code} %{size_download} %{time_starttransfer} %{time_total}\n"]);
        for _ in 0..21 {
            command.args(["--output", "/dev/null", &url]);
        }
        let output = command.output().await?;
        if !output.status.success() {
            return Err(format!("curl failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }
        let text = String::from_utf8(output.stdout)?;
        let mut totals = Vec::new();
        let mut headers = Vec::new();
        for (index, line) in text.lines().enumerate() {
            let fields: Vec<_> = line.split_whitespace().collect();
            if fields.len() != 5
                || fields[0] != expected
                || fields[1] != "200"
                || fields[2] != "59571"
            {
                return Err(format!("unexpected response: {line}").into());
            }
            if index != 0 {
                headers.push(fields[3].parse::<f64>()? * 1000.0);
                totals.push(fields[4].parse::<f64>()? * 1000.0);
            }
        }
        assert_eq!(totals.len(), 20);
        totals.sort_by(f64::total_cmp);
        headers.sort_by(f64::total_cmp);
        tracing::info!(
            tcp_nodelay = nodelay,
            http_version = expected,
            samples = 20,
            ttfb_median_ms = (headers[9] + headers[10]) / 2.0,
            total_median_ms = (totals[9] + totals[10]) / 2.0,
            total_p95_ms = totals[18],
            "Reused loopback TLS connection latency"
        );
    }
    Ok(())
}
