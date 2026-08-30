//! Deterministic in-process executor for portable smoke measurements.

use std::hint::black_box;

use crate::{
    config::RequestSpec,
    executor::types::{ExecutionOutcome, RequestFailure},
};

#[derive(Clone, Copy, Debug, Default)]
pub struct FixtureExecutor;

impl FixtureExecutor {
    pub fn execute(&self, request: &RequestSpec) -> Result<ExecutionOutcome, RequestFailure> {
        let mut checksum = hash_bytes(request.method.as_bytes());
        checksum ^= hash_bytes(request.path.as_bytes());
        for round in 0..request.fixture_work_units {
            checksum ^= round.wrapping_mul(0x9e37_79b9_7f4a_7c15);
            checksum = checksum.rotate_left(13);
            checksum = checksum.wrapping_mul(0xbf58_476d_1ce4_e5b9);
        }
        let checksum = black_box(checksum);
        Ok(ExecutionOutcome {
            status: request.expected_status,
            response_bytes: request.path.len().saturating_add(32),
            checksum,
        })
    }
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}
