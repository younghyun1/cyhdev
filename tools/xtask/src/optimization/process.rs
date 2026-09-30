//! Bounded driver processes and source identity checks.

use std::{
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use crate::{TaskError, TaskResult};

pub(super) fn output(command: &mut Command) -> TaskResult<String> {
    let result = match command.output() {
        Ok(result) => result,
        Err(error) => return Err(TaskError(format!("cannot start command: {error}"))),
    };
    if !result.status.success() {
        return Err(TaskError(format!("command failed with {}", result.status)));
    }
    match String::from_utf8(result.stdout) {
        Ok(value) => Ok(value.trim().into()),
        Err(error) => Err(TaskError(format!(
            "command returned invalid UTF-8: {error}"
        ))),
    }
}

/// Clean commits make every profile reproducible without copying private files.
pub(super) fn revision(root: &Path) -> TaskResult<String> {
    let status = output(
        Command::new("git")
            .args(["status", "--porcelain", "--untracked-files=normal"])
            .current_dir(root),
    )?;
    if !status.is_empty() {
        return Err(TaskError(
            "optimization requires a clean committed checkout".into(),
        ));
    }
    crate::eu5_web::require_pinned_checkout(root)?;
    output(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(root),
    )
}

/// Kill the driver's process group on timeout; its server must never remain running.
#[cfg(unix)]
pub(super) fn bounded(command: &mut Command, timeout_seconds: u64) -> TaskResult<()> {
    use std::os::unix::process::CommandExt;
    command.process_group(0).stdin(Stdio::null());
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => return Err(TaskError(format!("cannot start campaign driver: {error}"))),
    };
    let deadline = Instant::now() + Duration::from_secs(timeout_seconds);
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => {
                terminate_group(child.id())?;
                return Ok(());
            }
            Ok(Some(status)) => {
                terminate_group(child.id())?;
                return Err(TaskError(format!("campaign driver failed with {status}")));
            }
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(100)),
            result => {
                // Signal only the group created above, never an existing service.
                let signal = terminate_group(child.id());
                let kill_result = child.kill();
                let wait_result = child.wait();
                return Err(TaskError(format!(
                    "driver timeout or wait failure: {result:?}; group cleanup={signal:?}; child cleanup={kill_result:?}; reap={wait_result:?}"
                )));
            }
        }
    }
}

#[cfg(unix)]
fn terminate_group(child_id: u32) -> TaskResult<()> {
    let pid = match i32::try_from(child_id) {
        Ok(pid) => pid,
        Err(error) => return Err(TaskError(format!("invalid child process ID: {error}"))),
    };
    // The child starts a fresh process group; this never addresses a preexisting service.
    let result = unsafe { libc::kill(-pid, libc::SIGKILL) };
    if result == 0 {
        return Ok(());
    }
    let error = std::io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ESRCH) {
        Ok(())
    } else {
        Err(TaskError(format!(
            "cannot clean up driver process group: {error}"
        )))
    }
}
