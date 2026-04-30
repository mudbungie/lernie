//! CLI outbound: the frontend's sole command surface to the harness.
//!
//! Per ARCHITECTURE.md §3.4 ("CLI as control plane") and §3.5 ("UI
//! contract"), every user action a frontend issues — new prompt, stop,
//! fork-from-history — is an `exec("lernie", args)` and nothing else.
//! (Per the §2.9 amendment landed in bl-abf3, "resume" is no longer a
//! user-facing operation; continuing from a stopped branch is `lernie
//! prompt` or fork-from-history.) This module wraps that exec with
//! stream-chunked stdout/stderr, terminal exit reporting, and
//! aggressive SIGTERM-then-SIGKILL cleanup on drop (§2.9: "Stops are
//! aggressive").
//!
//! The module is pure Rust — no egui/eframe dependency — so a future
//! `lernie-ui-web` crate can reuse it unchanged. It has no knowledge of
//! specific subcommands: the caller supplies argv and consumes the stream.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

const TERM_GRACE: Duration = Duration::from_millis(500);
const READ_BUF: usize = 4096;

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("failed to spawn {binary}: {source}")]
    Spawn {
        binary: PathBuf,
        source: std::io::Error,
    },
}

/// One piece of output from a running `lernie` subprocess. The final
/// chunk in any stream is always `Exited`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Chunk {
    Stdout(Vec<u8>),
    Stderr(Vec<u8>),
    Exited(ExitInfo),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitInfo {
    Code(i32),
    Signal(i32),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cli {
    binary: PathBuf,
}

impl Cli {
    pub fn new(binary: impl Into<PathBuf>) -> Self {
        Self {
            binary: binary.into(),
        }
    }

    /// Resolve the `lernie` binary: `LERNIE_BINARY` if set and non-empty,
    /// otherwise `"lernie"` (looked up on `PATH` by the OS at exec time).
    pub fn resolve() -> Self {
        match std::env::var_os("LERNIE_BINARY") {
            Some(v) if !v.is_empty() => Self::new(PathBuf::from(v)),
            _ => Self::new("lernie"),
        }
    }

    pub fn binary(&self) -> &Path {
        &self.binary
    }

    /// Spawn `lernie <args...>` and return a streaming handle. Stdout and
    /// stderr are piped; stdin is closed. Dropping the returned `Stream`
    /// terminates the child (SIGTERM, then SIGKILL after a short grace).
    pub fn run(&self, args: &[&str]) -> Result<Stream, CliError> {
        let mut child = Command::new(&self.binary)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| CliError::Spawn {
                binary: self.binary.clone(),
                source: e,
            })?;
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let (tx, rx) = mpsc::channel();
        let tx_err = tx.clone();
        thread::spawn(move || pump(stdout, tx, Chunk::Stdout));
        thread::spawn(move || pump(stderr, tx_err, Chunk::Stderr));
        Ok(Stream {
            child: Some(child),
            rx,
            exit_emitted: false,
        })
    }
}

fn pump<R: Read>(mut reader: R, tx: Sender<Chunk>, wrap: fn(Vec<u8>) -> Chunk) {
    let mut buf = [0u8; READ_BUF];
    while pump_step(&mut reader, &tx, &mut buf, wrap) {}
}

fn pump_step<R: Read>(
    reader: &mut R,
    tx: &Sender<Chunk>,
    buf: &mut [u8],
    wrap: fn(Vec<u8>) -> Chunk,
) -> bool {
    match reader.read(buf) {
        Ok(0) | Err(_) => false,
        Ok(n) => tx.send(wrap(buf[..n].to_vec())).is_ok(),
    }
}

/// Live handle to a running subprocess. Iterate to consume chunks;
/// `Exited` is always the final item. Drop to terminate early.
pub struct Stream {
    child: Option<Child>,
    rx: Receiver<Chunk>,
    exit_emitted: bool,
}

impl Stream {
    pub fn pid(&self) -> Option<u32> {
        self.child.as_ref().map(|c| c.id())
    }
}

impl Iterator for Stream {
    type Item = Chunk;

    fn next(&mut self) -> Option<Chunk> {
        if self.exit_emitted {
            return None;
        }
        match self.rx.recv() {
            Ok(chunk) => Some(chunk),
            Err(_) => {
                self.exit_emitted = true;
                let status = self.child.take().and_then(|mut c| c.wait().ok());
                Some(Chunk::Exited(exit_info(status)))
            }
        }
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        let Some(mut child) = self.child.take() else {
            return;
        };
        #[allow(clippy::cast_possible_wrap)]
        let pid = child.id() as i32;
        unsafe { libc::kill(pid, libc::SIGTERM) };
        let deadline = Instant::now() + TERM_GRACE;
        while Instant::now() < deadline {
            if let Ok(Some(_)) = child.try_wait() {
                return;
            }
            thread::sleep(Duration::from_millis(25));
        }
        let _ = child.kill();
        let _ = child.wait();
    }
}

fn exit_info(status: Option<std::process::ExitStatus>) -> ExitInfo {
    use std::os::unix::process::ExitStatusExt;
    status
        .map(|s| match (s.code(), s.signal()) {
            (Some(c), _) => ExitInfo::Code(c),
            (_, Some(sig)) => ExitInfo::Signal(sig),
            _ => ExitInfo::Unknown,
        })
        .unwrap_or(ExitInfo::Unknown)
}

#[cfg(test)]
mod tests;
