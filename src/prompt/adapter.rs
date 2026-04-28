//! Harness-side of the provider-adapter contract (ARCH §4.4).
//!
//! The harness invokes `lernie-provider-<name>` as a subprocess per
//! model call, pipes a Messages-API request to its stdin, and reads the
//! adapter's stdout as **JSON Lines** — one event per line for
//! streaming `complete`, one line for `describe`. [`AdapterRunner`] is
//! the trait [`super::run`] depends on; [`SpawnAdapter`] is the
//! production implementation.
//!
//! Per-line dispatch is structural: the §4.4 streaming wire emits one
//! event per line, and the harness is the live writer of
//! `<conv-repo>/steps/<conv-id>/<NNN>/response.json` (§3.5). Routing
//! lines through a callback lets the harness append each event to disk
//! as it arrives and feed the in-memory assembler in the same pass —
//! and lets `describe` reuse the same path with a one-line collector.
//!
//! Child env is inherited by default; the harness layers explicit
//! key/value pairs on top via the `envs` argument. That covers the §4.4
//! `endpoint_env` handoff (harness-set values from `providers.yaml`)
//! while leaving credential vars (`auth_env`) to inherit naturally —
//! aligning with §4.4 "auth lives entirely inside the adapter."

use std::ffi::OsString;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;

/// Subdirectory under the harness root where provider adapter binaries
/// live (ARCH §4.1, §4.4 — "the harness looks up
/// `lernie-provider-<name>` at `<harness-root>/adapters/` … before
/// falling back to `PATH`"). Mirrors the `<harness-root>/tools/`
/// resolution path for tool binaries (§3.3).
pub const ADAPTERS_DIR: &str = "adapters";

/// Name prefix for provider adapter binaries (ARCH §4.4).
pub const ADAPTER_PREFIX: &str = "lernie-provider-";

/// Resolve the adapter binary for `provider_name`. Apply the §4.4
/// resolution order: try `<harness_root>/adapters/lernie-provider-<name>`
/// first; if absent, return the bare name so [`Command`] can resolve
/// against `PATH`. The harness-root copy wins so per-install adapter
/// rotations stay local to one harness root.
pub fn resolve_binary(harness_root: &Path, provider_name: &str) -> OsString {
    let bare = format!("{ADAPTER_PREFIX}{provider_name}");
    let harness_path = harness_root.join(ADAPTERS_DIR).join(&bare);
    if harness_path.is_file() {
        return harness_path.into_os_string();
    }
    OsString::from(bare)
}

/// The slice of the adapter contract that the harness calls into.
///
/// One subprocess per [`Self::run`]. As the child writes stdout, every
/// completed line (terminator stripped, blanks skipped) is handed to
/// `on_line`. The callback may surface an [`io::Error`] to abort early;
/// otherwise the call returns when the child exits and stdout reaches
/// EOF. Non-zero exit surfaces as an error because §4.4 reserves that
/// for adapter-side crashes, not in-band provider failures.
pub trait AdapterRunner {
    /// Spawn `binary` with `args` and `envs`, write `stdin_bytes` to its
    /// stdin (closing it after), and route each stdout line through
    /// `on_line`.
    fn run(
        &self,
        binary: &OsString,
        args: &[&str],
        envs: &[(&str, &str)],
        stdin_bytes: &[u8],
        on_line: &mut dyn FnMut(&[u8]) -> io::Result<()>,
    ) -> io::Result<()>;
}

/// Default [`AdapterRunner`]. Uses [`Command`] with PATH lookup.
#[derive(Debug, Clone, Copy)]
pub struct SpawnAdapter;

impl AdapterRunner for SpawnAdapter {
    fn run(
        &self,
        binary: &OsString,
        args: &[&str],
        envs: &[(&str, &str)],
        stdin_bytes: &[u8],
        on_line: &mut dyn FnMut(&[u8]) -> io::Result<()>,
    ) -> io::Result<()> {
        let mut child = Command::new(binary)
            .args(args)
            .envs(envs.iter().copied())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        // Stdin in a thread so a slow / large request never deadlocks
        // against the child's stdout pipe-buffer fill (the harness
        // tails stdout on the main thread).
        let mut stdin = child.stdin.take().expect("stdin is piped");
        let stdin_owned = stdin_bytes.to_vec();
        let stdin_thread = thread::spawn(move || -> io::Result<()> {
            stdin.write_all(&stdin_owned)?;
            // Drop closes the fd, signaling EOF.
            Ok(())
        });

        let stdout = child.stdout.take().expect("stdout is piped");
        let mut reader = BufReader::new(stdout);
        let mut buf = Vec::new();
        loop {
            buf.clear();
            let n = reader.read_until(b'\n', &mut buf)?;
            if n == 0 {
                break;
            }
            let line = strip_trailing_lf(&buf);
            if line.is_empty() {
                continue;
            }
            on_line(line)?;
        }

        // Surface any stdin-thread failure as an io::Error before we
        // claim success. A panic in the thread is itself a harness
        // fault — propagate via expect rather than swallowing.
        stdin_thread.join().expect("stdin writer thread panicked")?;
        let status = child.wait()?;
        if !status.success() {
            let mut stderr = String::new();
            if let Some(mut s) = child.stderr.take() {
                use std::io::Read;
                let _ = s.read_to_string(&mut stderr);
            }
            return Err(io::Error::other(format!(
                "adapter {:?} exited with {}: {}",
                binary,
                status,
                stderr.trim()
            )));
        }
        Ok(())
    }
}

/// Strip a single trailing `\n` (and the `\r` of a `\r\n` pair) from
/// `buf` so callbacks see clean payload bytes.
fn strip_trailing_lf(buf: &[u8]) -> &[u8] {
    let trimmed = buf.strip_suffix(b"\n").unwrap_or(buf);
    trimmed.strip_suffix(b"\r").unwrap_or(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Drain `runner` into a single buffer of all line payloads, one
    /// per `\n`. Used by the SpawnAdapter unit tests so each case stays
    /// focused on the spawn / env / exit behavior rather than callback
    /// plumbing.
    fn collect_lines<R: AdapterRunner>(
        runner: &R,
        bin: &OsString,
        args: &[&str],
        envs: &[(&str, &str)],
        stdin: &[u8],
    ) -> io::Result<Vec<Vec<u8>>> {
        let mut lines: Vec<Vec<u8>> = Vec::new();
        runner.run(bin, args, envs, stdin, &mut |line| {
            lines.push(line.to_vec());
            Ok(())
        })?;
        Ok(lines)
    }

    #[test]
    fn resolve_binary_prefers_harness_root_when_present() {
        let tmp = tempfile::tempdir().unwrap();
        let adapters = tmp.path().join(ADAPTERS_DIR);
        std::fs::create_dir_all(&adapters).unwrap();
        let installed = adapters.join("lernie-provider-anthropic");
        std::fs::write(&installed, b"#!/bin/sh\nexit 0\n").unwrap();

        let resolved = resolve_binary(tmp.path(), "anthropic");
        assert_eq!(PathBuf::from(&resolved), installed);
    }

    #[test]
    fn resolve_binary_falls_back_to_bare_name_for_path_lookup() {
        let tmp = tempfile::tempdir().unwrap();
        // No adapters/ subdir — resolver must hand back the bare name
        // so Command's PATH lookup runs.
        let resolved = resolve_binary(tmp.path(), "anthropic");
        assert_eq!(resolved, OsString::from("lernie-provider-anthropic"));
    }

    #[test]
    fn resolve_binary_falls_back_when_path_is_a_directory() {
        // A directory at the expected adapter path doesn't count as the
        // adapter binary — must fall back to PATH.
        let tmp = tempfile::tempdir().unwrap();
        let bogus = tmp.path().join(ADAPTERS_DIR).join("lernie-provider-acme");
        std::fs::create_dir_all(&bogus).unwrap();

        let resolved = resolve_binary(tmp.path(), "acme");
        assert_eq!(resolved, OsString::from("lernie-provider-acme"));
    }

    #[test]
    fn spawn_adapter_emits_one_callback_per_stdout_line() {
        // `printf` writes three newline-separated lines. The runner
        // should hand them off one at a time, terminator stripped, and
        // skip the trailing blank.
        let bin = OsString::from("printf");
        let lines = collect_lines(&SpawnAdapter, &bin, &["a\nbb\nccc\n"], &[], b"").unwrap();
        assert_eq!(lines, vec![b"a".to_vec(), b"bb".to_vec(), b"ccc".to_vec()]);
    }

    #[test]
    fn spawn_adapter_handles_crlf_terminators() {
        // CRLF terminators show up when an adapter is built on a
        // Windows-y stdlib. Strip both the \r and the \n.
        let bin = OsString::from("printf");
        let lines = collect_lines(&SpawnAdapter, &bin, &["x\r\ny\r\n"], &[], b"").unwrap();
        assert_eq!(lines, vec![b"x".to_vec(), b"y".to_vec()]);
    }

    #[test]
    fn spawn_adapter_pipes_stdin_to_child() {
        // `cat` copies stdin to stdout; the runner reads that as one
        // line of output.
        let bin = OsString::from("cat");
        let lines = collect_lines(&SpawnAdapter, &bin, &[], &[], b"hello\n").unwrap();
        assert_eq!(lines, vec![b"hello".to_vec()]);
    }

    #[test]
    fn spawn_adapter_forwards_envs_to_child() {
        // `env -0` prints the child env, NUL-terminated. We can scan
        // for the var we set without picking up the inherited
        // PATH/USER lines.
        let bin = OsString::from("env");
        let mut seen = String::new();
        SpawnAdapter
            .run(
                &bin,
                &["-0"],
                &[("LERNIE_TEST_VAR", "passthrough-ok")],
                b"",
                &mut |line| {
                    seen.push_str(&String::from_utf8_lossy(line));
                    seen.push('\n');
                    Ok(())
                },
            )
            .unwrap();
        assert!(
            seen.split('\0')
                .any(|kv| kv == "LERNIE_TEST_VAR=passthrough-ok"),
            "env not forwarded; got: {seen}"
        );
    }

    #[test]
    fn spawn_adapter_reports_spawn_failure() {
        let bin = OsString::from("/no/such/lernie-provider-nonesuch");
        let err = collect_lines(&SpawnAdapter, &bin, &[], &[], b"").unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn spawn_adapter_reports_nonzero_exit() {
        // `false` exits with status 1 on every POSIX system.
        let bin = OsString::from("false");
        let err = collect_lines(&SpawnAdapter, &bin, &[], &[], b"").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("exited with"), "got: {msg}");
    }

    #[test]
    fn spawn_adapter_propagates_callback_error() {
        // A callback that errors aborts the run — the runner surfaces
        // the io::Error rather than swallowing it.
        let bin = OsString::from("printf");
        let err = SpawnAdapter
            .run(&bin, &["one\ntwo\n"], &[], b"", &mut |_| {
                Err(io::Error::other("callback bailed"))
            })
            .unwrap_err();
        assert!(err.to_string().contains("callback bailed"));
    }
}
