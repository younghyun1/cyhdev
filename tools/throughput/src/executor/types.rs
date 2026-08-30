//! Bounded request execution results shared by fixture and HTTP modes.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutorKind {
    Fixture,
    Http,
}

impl ExecutorKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::Http => "http",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ExecutionOutcome {
    pub status: u16,
    pub response_bytes: usize,
    pub checksum: u64,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RequestFailure {
    Connect,
    ConfigureSocket,
    Timeout,
    Write,
    Read,
    ResponseTooLarge,
    InvalidResponse,
    UnexpectedStatus,
}

impl RequestFailure {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Connect => "connect",
            Self::ConfigureSocket => "configure_socket",
            Self::Timeout => "timeout",
            Self::Write => "write",
            Self::Read => "read",
            Self::ResponseTooLarge => "response_too_large",
            Self::InvalidResponse => "invalid_response",
            Self::UnexpectedStatus => "unexpected_status",
        }
    }
}
