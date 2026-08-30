//! Executor selection without changing the recorded request schedule.

use std::time::Duration;

use crate::{
    config::{RequestSpec, WorkloadConfig},
    error::HarnessResult,
    executor::{
        fixture::FixtureExecutor,
        http::HttpExecutor,
        types::{ExecutionOutcome, ExecutorKind, RequestFailure},
    },
};

#[derive(Clone, Debug)]
pub enum Executor {
    Fixture(FixtureExecutor),
    Http(HttpExecutor),
}

impl Executor {
    pub fn new(target: Option<&str>, workload: &WorkloadConfig) -> HarnessResult<Self> {
        match target {
            Some(target) => Ok(Self::Http(HttpExecutor::new(
                target,
                Duration::from_millis(workload.timeout_ms),
                workload.max_response_bytes,
            )?)),
            None => Ok(Self::Fixture(FixtureExecutor)),
        }
    }

    pub fn execute(&self, request: &RequestSpec) -> Result<ExecutionOutcome, RequestFailure> {
        let outcome = match self {
            Self::Fixture(executor) => executor.execute(request),
            Self::Http(executor) => executor.execute(request),
        }?;
        if outcome.status == request.expected_status {
            Ok(outcome)
        } else {
            Err(RequestFailure::UnexpectedStatus)
        }
    }

    pub const fn kind(&self) -> ExecutorKind {
        match self {
            Self::Fixture(_) => ExecutorKind::Fixture,
            Self::Http(_) => ExecutorKind::Http,
        }
    }

    pub fn label(&self) -> String {
        match self {
            Self::Fixture(_) => "in-process deterministic fixture".to_owned(),
            Self::Http(executor) => executor.label(),
        }
    }

    pub fn resolved_address(&self) -> Option<String> {
        match self {
            Self::Fixture(_) => None,
            Self::Http(executor) => Some(executor.resolved_address()),
        }
    }
}
