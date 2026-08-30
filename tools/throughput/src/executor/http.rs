//! Plain-HTTP executor for an explicitly supplied local or staged target.

use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpStream, ToSocketAddrs},
    time::{Duration, Instant},
};

use crate::{
    config::RequestSpec,
    error::{HarnessError, HarnessResult},
    executor::http_target::parse_target,
    executor::types::{ExecutionOutcome, RequestFailure},
};

#[derive(Clone, Debug)]
pub struct HttpExecutor {
    address: SocketAddr,
    authority: String,
    base_path: String,
    timeout: Duration,
    max_response_bytes: usize,
}

impl HttpExecutor {
    pub fn new(raw_target: &str, timeout: Duration, max_response_bytes: usize) -> HarnessResult<Self> {
        let target = parse_target(raw_target)?;
        let mut addresses = (target.host.as_str(), target.port)
            .to_socket_addrs()
            .map_err(|source| HarnessError::Resolve {
                target: raw_target.to_owned(),
                source,
            })?;
        let address = match addresses.next() {
            Some(address) => address,
            None => {
                return Err(HarnessError::Arguments(format!(
                    "HTTP target `{raw_target}` resolved to no addresses"
                )));
            }
        };
        Ok(Self {
            address,
            authority: target.authority,
            base_path: target.base_path,
            timeout,
            max_response_bytes,
        })
    }

    pub fn execute(&self, request: &RequestSpec) -> Result<ExecutionOutcome, RequestFailure> {
        let deadline = Instant::now()
            .checked_add(self.timeout)
            .ok_or(RequestFailure::Timeout)?;
        let mut stream = TcpStream::connect_timeout(&self.address, remaining(deadline)?)
            .map_err(|error| classify_io(&error, RequestFailure::Connect))?;

        let path = self.request_path(&request.path);
        let wire_request = format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\nAccept: application/json\r\nConnection: close\r\nUser-Agent: cyhdev-throughput/1\r\n\r\n",
            request.method, path, self.authority
        );
        write_before_deadline(&mut stream, wire_request.as_bytes(), deadline)?;

        let response = self.read_response(&mut stream, deadline)?;
        let status = parse_status(&response)?;
        let _remaining = remaining(deadline)?;
        Ok(ExecutionOutcome {
            status,
            response_bytes: response.len(),
            checksum: hash_bytes(&response),
        })
    }

    pub fn label(&self) -> String {
        if self.base_path == "/" {
            format!("http://{}", self.authority)
        } else {
            format!("http://{}{}", self.authority, self.base_path)
        }
    }

    pub fn resolved_address(&self) -> String {
        self.address.to_string()
    }

    fn request_path(&self, request_path: &str) -> String {
        if self.base_path == "/" {
            request_path.to_owned()
        } else {
            format!("{}{}", self.base_path.trim_end_matches('/'), request_path)
        }
    }

    fn read_response(
        &self,
        stream: &mut TcpStream,
        deadline: Instant,
    ) -> Result<Vec<u8>, RequestFailure> {
        let initial_capacity = self.max_response_bytes.min(8 * 1024);
        let mut response = Vec::with_capacity(initial_capacity);
        let mut chunk = [0_u8; 8 * 1024];
        loop {
            stream
                .set_read_timeout(Some(remaining(deadline)?))
                .map_err(|_source| RequestFailure::ConfigureSocket)?;
            let count = match stream.read(&mut chunk) {
                Ok(count) => count,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(classify_io(&error, RequestFailure::Read)),
            };
            if count == 0 {
                break;
            }
            let new_length = match response.len().checked_add(count) {
                Some(length) => length,
                None => return Err(RequestFailure::ResponseTooLarge),
            };
            if new_length > self.max_response_bytes {
                return Err(RequestFailure::ResponseTooLarge);
            }
            response.extend_from_slice(&chunk[..count]);
        }
        Ok(response)
    }
}

fn write_before_deadline(
    stream: &mut TcpStream,
    mut bytes: &[u8],
    deadline: Instant,
) -> Result<(), RequestFailure> {
    while !bytes.is_empty() {
        stream
            .set_write_timeout(Some(remaining(deadline)?))
            .map_err(|_source| RequestFailure::ConfigureSocket)?;
        let written = match stream.write(bytes) {
            Ok(0) => return Err(RequestFailure::Write),
            Ok(written) => written,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(classify_io(&error, RequestFailure::Write)),
        };
        bytes = &bytes[written..];
    }
    Ok(())
}

fn remaining(deadline: Instant) -> Result<Duration, RequestFailure> {
    match deadline.checked_duration_since(Instant::now()) {
        Some(remaining) if !remaining.is_zero() => Ok(remaining),
        Some(_) | None => Err(RequestFailure::Timeout),
    }
}

fn classify_io(error: &io::Error, fallback: RequestFailure) -> RequestFailure {
    if matches!(error.kind(), io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock) {
        RequestFailure::Timeout
    } else {
        fallback
    }
}

fn parse_status(response: &[u8]) -> Result<u16, RequestFailure> {
    let header_end = response
        .windows(2)
        .position(|window| window == b"\r\n")
        .ok_or(RequestFailure::InvalidResponse)?;
    let status_line = std::str::from_utf8(&response[..header_end])
        .map_err(|_source| RequestFailure::InvalidResponse)?;
    let mut fields = status_line.split_ascii_whitespace();
    let protocol = fields.next().ok_or(RequestFailure::InvalidResponse)?;
    let status = fields.next().ok_or(RequestFailure::InvalidResponse)?;
    if !protocol.starts_with("HTTP/1.") {
        return Err(RequestFailure::InvalidResponse);
    }
    status
        .parse::<u16>()
        .map_err(|_source| RequestFailure::InvalidResponse)
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use std::{io::Write, net::TcpListener, thread, time::Duration};

    use crate::{config::RequestSpec, executor::types::RequestFailure};

    use super::HttpExecutor;

    #[test]
    fn canonical_root_target_matches_environment_template() -> Result<(), String> {
        let executor = HttpExecutor::new(
            "http://127.0.0.1:3000",
            Duration::from_secs(1),
            1_024,
        ).map_err(|error| error.to_string())?;
        assert_eq!(executor.label(), "http://127.0.0.1:3000");
        Ok(())
    }

    #[test]
    fn enforces_one_end_to_end_deadline() -> Result<(), String> {
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|error| error.to_string())?;
        let address = listener.local_addr().map_err(|error| error.to_string())?;
        let server = thread::spawn(move || {
            if let Ok((mut stream, _peer)) = listener.accept() {
                thread::sleep(Duration::from_millis(40));
                let _result = stream.write_all(b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n");
            }
        });
        let executor = HttpExecutor::new(
            &format!("http://{address}"),
            Duration::from_millis(10),
            1_024,
        ).map_err(|error| error.to_string())?;
        let request = RequestSpec {
            name: "deadline".to_owned(), method: "GET".to_owned(), path: "/".to_owned(),
            expected_status: 200, weight: 1, fixture_work_units: 1,
        };
        let outcome = executor.execute(&request);
        server.join().map_err(|_payload| "deadline test server panicked".to_owned())?;
        if matches!(outcome, Err(RequestFailure::Timeout)) {
            Ok(())
        } else {
            Err(format!("expected deadline failure, got {outcome:?}"))
        }
    }
}
