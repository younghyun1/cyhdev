//! Cleanup exclusively owned subprocess groups and the newly created fixture container.

use super::{bootstrap_inputs, process};
use crate::{TaskError, TaskResult};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

static INTERRUPTED: AtomicBool = AtomicBool::new(false);
pub(super) fn interrupted() -> bool {
    INTERRUPTED.load(Ordering::Relaxed)
}

extern "C" fn interrupt(_: libc::c_int) {
    INTERRUPTED.store(true, Ordering::Relaxed);
}

pub(super) struct Signals {
    previous: Vec<(i32, libc::sigaction)>,
}
impl Signals {
    pub fn install() -> TaskResult<Self> {
        INTERRUPTED.store(false, Ordering::Relaxed);
        let mut signals = Self {
            previous: Vec::new(),
        };
        for signal in [libc::SIGINT, libc::SIGTERM] {
            // A zeroed sigaction is initialized before registration; the handler only writes an atomic.
            let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
            action.sa_sigaction = interrupt as *const () as usize;
            unsafe {
                libc::sigemptyset(&mut action.sa_mask);
            }
            let mut previous = unsafe { std::mem::zeroed() };
            if unsafe { libc::sigaction(signal, &action, &mut previous) } != 0 {
                return Err(TaskError(format!(
                    "cannot install build cancellation handler: {}",
                    std::io::Error::last_os_error()
                )));
            }
            signals.previous.push((signal, previous));
        }
        Ok(signals)
    }
}
impl Drop for Signals {
    fn drop(&mut self) {
        for (signal, previous) in &self.previous {
            if unsafe { libc::sigaction(*signal, previous, std::ptr::null_mut()) } != 0 {
                eprintln!(
                    "cannot restore cancellation handler: {}",
                    std::io::Error::last_os_error()
                );
            }
        }
    }
}

pub(super) struct OwnedChild {
    child: Child,
    finished: bool,
}
impl OwnedChild {
    pub fn spawn(command: &mut Command) -> TaskResult<Self> {
        use std::os::unix::process::CommandExt;
        command.process_group(0).stdin(Stdio::null());
        match command.spawn() {
            Ok(child) => Ok(Self {
                child,
                finished: false,
            }),
            Err(error) => Err(TaskError(format!("cannot start build subprocess: {error}"))),
        }
    }
    pub fn alive(&mut self) -> TaskResult<bool> {
        match self.child.try_wait() {
            Ok(None) => Ok(true),
            Ok(Some(_)) => {
                self.finished = true;
                Ok(false)
            }
            Err(error) => Err(TaskError(format!(
                "cannot inspect build subprocess: {error}"
            ))),
        }
    }
    pub fn wait(&mut self) -> TaskResult<()> {
        loop {
            if interrupted() {
                return Err(TaskError(
                    "optimization build interrupted; stopping its own fixtures".into(),
                ));
            }
            match self.child.try_wait() {
                Ok(None) => thread::sleep(Duration::from_millis(100)),
                Ok(Some(status)) => {
                    self.finished = true;
                    if status.success() {
                        return Ok(());
                    }
                    return Err(TaskError(format!(
                        "optimization subprocess failed with {status}"
                    )));
                }
                Err(error) => {
                    return Err(TaskError(format!(
                        "cannot wait for build subprocess: {error}"
                    )));
                }
            }
        }
    }
    fn stop(&mut self) -> TaskResult<()> {
        if self.finished {
            return Ok(());
        }
        let pid = i32::try_from(self.child.id())
            .map_err(|error| TaskError(format!("invalid owned process ID: {error}")))?;
        // The group was created by spawn above, never discovered from a port or process name.
        unsafe {
            libc::kill(-pid, libc::SIGTERM);
        }
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            if !self.alive()? {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(50));
        }
        unsafe {
            libc::kill(-pid, libc::SIGKILL);
        }
        self.child
            .wait()
            .map_err(|error| TaskError(format!("cannot reap owned subprocess: {error}")))?;
        self.finished = true;
        Ok(())
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if let Err(error) = self.stop() {
            eprintln!("build subprocess cleanup failed: {error}");
        }
    }
}

pub(super) struct Database {
    root: PathBuf,
    cidfile: PathBuf,
}
impl Database {
    pub fn new(root: &Path, runtime: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            cidfile: runtime.join("postgres.cid"),
        }
    }
    pub fn start_command(&self, name: &str, image: &str) -> Command {
        let mut command = bootstrap_inputs::command("docker", &self.root);
        command
            .args(["run", "--detach", "--rm", "--name", name, "--cidfile"])
            .arg(&self.cidfile)
            .args([
                "--publish",
                "127.0.0.1:35432:5432",
                "--tmpfs",
                "/var/lib/postgresql:rw,nosuid,size=1g",
                "--memory",
                "1g",
                "--shm-size",
                "128m",
                "--env",
                "POSTGRES_USER=optimization_fixture",
                "--env",
                "POSTGRES_PASSWORD=optimization-fixture-postgres",
                "--env",
                "POSTGRES_DB=postgres",
                image,
                "-c",
                "max_wal_size=256MB",
                "-c",
                "min_wal_size=64MB",
            ]);
        command
    }
    pub fn cleanup(&self) -> TaskResult<()> {
        let metadata = match fs::symlink_metadata(&self.cidfile) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => {
                return Err(TaskError(format!(
                    "cannot inspect fixture container receipt: {error}"
                )));
            }
            Ok(metadata) => metadata,
        };
        if !metadata.is_file() || metadata.len() > 65 {
            return Err(TaskError(
                "invalid fixture container receipt; refusing cleanup".into(),
            ));
        }
        let id = fs::read_to_string(&self.cidfile).map_err(|error| {
            TaskError(format!("cannot read fixture container receipt: {error}"))
        })?;
        let id = id.trim();
        if id.len() != 64 || !id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(TaskError(
                "invalid fixture container identity; refusing cleanup".into(),
            ));
        }
        process::output(
            bootstrap_inputs::command("docker", &self.root).args(["rm", "--force", id]),
        )?;
        fs::remove_file(&self.cidfile)
            .map_err(|error| TaskError(format!("cannot remove fixture container receipt: {error}")))
    }
}
impl Drop for Database {
    fn drop(&mut self) {
        if let Err(error) = self.cleanup() {
            eprintln!("fixture database cleanup failed: {error}");
        }
    }
}
