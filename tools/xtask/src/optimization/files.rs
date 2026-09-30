//! Size-limited input and streaming integrity checks.

use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};

use crate::{TaskError, TaskResult};

pub(super) fn read_json<T: DeserializeOwned>(path: &Path, limit: u64) -> TaskResult<T> {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) => {
            return Err(TaskError(format!(
                "cannot open {}: {error}",
                path.display()
            )));
        }
    };
    let mut bytes = Vec::new();
    if let Err(error) = file.take(limit + 1).read_to_end(&mut bytes) {
        return Err(TaskError(format!(
            "cannot read {}: {error}",
            path.display()
        )));
    }
    if bytes.len() as u64 > limit {
        return Err(TaskError(format!(
            "{} exceeds {limit} bytes",
            path.display()
        )));
    }
    match serde_json::from_slice(&bytes) {
        Ok(value) => Ok(value),
        Err(error) => Err(TaskError(format!(
            "invalid JSON in {}: {error}",
            path.display()
        ))),
    }
}

pub(super) fn digest(path: &Path) -> TaskResult<String> {
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) => {
            return Err(TaskError(format!(
                "cannot hash {}: {error}",
                path.display()
            )));
        }
    };
    let mut digest = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        match file.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => digest.update(&buffer[..count]),
            Err(error) => {
                return Err(TaskError(format!(
                    "cannot hash {}: {error}",
                    path.display()
                )));
            }
        }
    }
    Ok(digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

pub(super) fn write_json(path: &Path, value: &impl Serialize) -> TaskResult<()> {
    let bytes = match serde_json::to_vec_pretty(value) {
        Ok(bytes) => bytes,
        Err(error) => return Err(TaskError(format!("cannot serialize receipt: {error}"))),
    };
    let temporary = path.with_extension("json.tmp");
    let mut file = match fs::File::create(&temporary) {
        Ok(file) => file,
        Err(error) => return Err(TaskError(format!("cannot create receipt: {error}"))),
    };
    if let Err(error) = file.write_all(&bytes).and_then(|()| file.sync_all()) {
        return Err(TaskError(format!("cannot save receipt: {error}")));
    }
    match fs::rename(temporary, path) {
        Ok(()) => Ok(()),
        Err(error) => Err(TaskError(format!("cannot publish receipt: {error}"))),
    }
}

pub(super) fn copy(source: &Path, destination: &Path) -> TaskResult<()> {
    match fs::copy(source, destination) {
        Ok(_) => Ok(()),
        Err(error) => Err(TaskError(format!(
            "cannot copy {}: {error}",
            source.display()
        ))),
    }
}
