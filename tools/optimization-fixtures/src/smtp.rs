//! Bounded authenticated TLS mailbox; recipients are confined to example.test.

use base64::{Engine, engine::general_purpose::STANDARD};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::Semaphore,
};

pub async fn serve(runtime: PathBuf) -> anyhow::Result<()> {
    let cert = CertificateDer::from_pem_slice(&std::fs::read(runtime.join("tls-cert.pem"))?)?;
    let key = PrivateKeyDer::from_pem_slice(&std::fs::read(runtime.join("tls-key.pem"))?)?;
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)?;
    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(config));
    let listener =
        TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, crate::files::SMTP_PORT)).await?;
    let admission = Arc::new(Semaphore::new(16));
    loop {
        let (socket, _) = listener.accept().await?;
        let permit = match admission.clone().try_acquire_owned() {
            Ok(p) => p,
            Err(_) => continue,
        };
        let (acceptor, runtime) = (acceptor.clone(), runtime.clone());
        tokio::spawn(async move {
            let result = tokio::time::timeout(Duration::from_secs(15), async {
                let stream = acceptor.accept(socket).await?;
                conversation(stream, runtime).await
            })
            .await;
            if !matches!(result, Ok(Ok(()))) {
                eprintln!("synthetic SMTP connection failed");
            }
            drop(permit);
        });
    }
}

async fn conversation(
    stream: impl AsyncRead + AsyncWrite + Unpin,
    runtime: PathBuf,
) -> anyhow::Result<()> {
    let mut stream = BufReader::new(stream);
    stream
        .write_all(b"220 localhost synthetic SMTP ready\r\n")
        .await?;
    let mut authenticated = false;
    let mut recipients = Vec::new();
    for _ in 0..128 {
        let line = bounded_line(&mut stream, 4096).await?;
        let trimmed = line.trim();
        let upper = trimmed.to_ascii_uppercase();
        let response = if upper.starts_with("EHLO ") || upper.starts_with("HELO ") {
            "250-localhost\r\n250-AUTH PLAIN LOGIN\r\n250 SIZE 524288\r\n"
        } else if upper.starts_with("AUTH PLAIN") {
            let token = match trimmed.split_whitespace().nth(2) {
                Some(token) => token.to_owned(),
                None => {
                    stream.write_all(b"334 \r\n").await?;
                    bounded_line(&mut stream, 4096).await?.trim().to_owned()
                }
            };
            let bytes = STANDARD.decode(token)?;
            let parts: Vec<&[u8]> = bytes.split(|byte| *byte == 0).collect();
            authenticated = parts.len() == 3
                && parts[1].starts_with(b"optimization-fixture-")
                && parts[2].starts_with(b"optimization-fixture-");
            if authenticated {
                "235 authenticated\r\n"
            } else {
                "535 rejected\r\n"
            }
        } else if upper == "AUTH LOGIN" {
            stream.write_all(b"334 VXNlcm5hbWU6\r\n").await?;
            let user = STANDARD.decode(bounded_line(&mut stream, 4096).await?.trim())?;
            stream.write_all(b"334 UGFzc3dvcmQ6\r\n").await?;
            let password = STANDARD.decode(bounded_line(&mut stream, 4096).await?.trim())?;
            authenticated = user.starts_with(b"optimization-fixture-")
                && password.starts_with(b"optimization-fixture-");
            if authenticated {
                "235 authenticated\r\n"
            } else {
                "535 rejected\r\n"
            }
        } else if upper.starts_with("MAIL FROM:") && authenticated {
            recipients.clear();
            "250 accepted\r\n"
        } else if upper.starts_with("RCPT TO:") && authenticated {
            let address = trimmed[8..]
                .trim()
                .trim_start_matches('<')
                .split('>')
                .next()
                .unwrap_or("");
            if address.ends_with("@example.test") && recipients.len() < 8 {
                recipients.push(address.to_owned());
                "250 accepted\r\n"
            } else {
                "550 synthetic recipients only\r\n"
            }
        } else if upper == "DATA" && authenticated && !recipients.is_empty() {
            stream.write_all(b"354 terminate with dot\r\n").await?;
            let mut data = Vec::new();
            loop {
                let line = bounded_line(&mut stream, 524288).await?;
                if line == ".\r\n" {
                    break;
                }
                anyhow::ensure!(
                    data.len() + line.len() <= 524288,
                    "mail exceeds fixture bounds"
                );
                data.extend_from_slice(line.as_bytes());
            }
            anyhow::ensure!(
                std::fs::read_dir(runtime.join("mail"))?.count() < 64,
                "mailbox full"
            );
            let mail =
                serde_json::json!({"recipients":recipients,"message":String::from_utf8(data)?});
            std::fs::write(
                runtime
                    .join("mail")
                    .join(format!("{}.json", uuid::Uuid::new_v4())),
                serde_json::to_vec(&mail)?,
            )?;
            "250 queued\r\n"
        } else if upper == "QUIT" {
            stream.write_all(b"221 closing\r\n").await?;
            return Ok(());
        } else if upper == "RSET" {
            recipients.clear();
            "250 reset\r\n"
        } else if upper == "NOOP" {
            "250 okay\r\n"
        } else {
            "503 invalid transaction\r\n"
        };
        stream.write_all(response.as_bytes()).await?;
    }
    anyhow::bail!("SMTP command budget exhausted")
}

async fn bounded_line<R: AsyncRead + Unpin>(
    reader: &mut BufReader<R>,
    limit: usize,
) -> anyhow::Result<String> {
    let mut bytes = Vec::new();
    loop {
        let chunk = reader.fill_buf().await?;
        anyhow::ensure!(!chunk.is_empty(), "SMTP input ended");
        let length = chunk
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(chunk.len(), |index| index + 1);
        anyhow::ensure!(bytes.len() + length <= limit, "SMTP line too large");
        let complete = chunk[length - 1] == b'\n';
        bytes.extend_from_slice(&chunk[..length]);
        reader.consume(length);
        if complete {
            return Ok(String::from_utf8(bytes)?);
        }
    }
}
