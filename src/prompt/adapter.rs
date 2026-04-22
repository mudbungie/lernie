//! Harness-side of the provider-adapter contract (ARCH §4.4).
//!
//! The harness invokes `lernie-provider-<name>` as a subprocess per model
//! call, pipes a Messages-API request to its stdin, and reads one JSON
//! document back on stdout. [`AdapterRunner`] is the trait [`super::run`]
//! depends on; [`SpawnAdapter`] is the production implementation.
//!
//! Child env is inherited by default: credential env vars (e.g.
//! `ANTHROPIC_API_KEY`) propagate without the harness having to read
//! `providers.yaml`'s `auth:` block — aligning with §4.4 "auth lives
//! entirely inside the adapter."

use std::ffi::OsString;
use std::io::{self, Write};
use std::process::{Command, Stdio};

/// The slice of the adapter contract that the harness calls into. One
/// subprocess per [`Self::run`]; non-zero exit surfaces as an error because
/// §4.4 reserves that for adapter-side crashes, not in-band provider
/// failures.
pub trait AdapterRunner {
    /// Spawn `binary` with `args`, forward `stdin_bytes` to its stdin,
    /// and return its stdout bytes on exit-zero. The caller is
    /// responsible for parsing the bytes as either an upstream response
    /// or an in-band adapter error (§4.4).
    fn run(&self, binary: &OsString, args: &[&str], stdin_bytes: &[u8]) -> io::Result<Vec<u8>>;
}

/// Default [`AdapterRunner`]. Uses [`Command`] with PATH lookup.
#[derive(Debug, Clone, Copy)]
pub struct SpawnAdapter;

impl AdapterRunner for SpawnAdapter {
    fn run(&self, binary: &OsString, args: &[&str], stdin_bytes: &[u8]) -> io::Result<Vec<u8>> {
        let mut child = Command::new(binary)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        {
            let mut stdin = child.stdin.take().expect("stdin is piped");
            stdin.write_all(stdin_bytes)?;
        }
        let out = child.wait_with_output()?;
        if !out.status.success() {
            return Err(io::Error::other(format!(
                "adapter {:?} exited with {}: {}",
                binary,
                out.status,
                String::from_utf8_lossy(&out.stderr).trim()
            )));
        }
        Ok(out.stdout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_adapter_runs_cat_as_echo() {
        // `cat` copies stdin to stdout — a portable stand-in for a real
        // adapter. Proves the happy path: args, stdin piping, stdout
        // capture, exit-zero.
        let bin = OsString::from("cat");
        let out = SpawnAdapter.run(&bin, &[], b"hello\n").unwrap();
        assert_eq!(out, b"hello\n");
    }

    #[test]
    fn spawn_adapter_reports_spawn_failure() {
        let bin = OsString::from("/no/such/lernie-provider-nonesuch");
        let err = SpawnAdapter.run(&bin, &[], b"").unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn spawn_adapter_reports_nonzero_exit() {
        // `false` exits with status 1 on every POSIX system.
        let bin = OsString::from("false");
        let err = SpawnAdapter.run(&bin, &[], b"").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("exited with"), "got: {msg}");
    }
}
