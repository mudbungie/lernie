//! Harness-side of the provider-adapter contract (ARCH §4.4).
//!
//! The provider adapter is brazen's `bz` — one stateless binary for
//! every provider. The harness invokes it **once per attempt** as
//! `bz --json --provider <row>`, pipes a canonical request (JSON) to its
//! stdin, and reads its stdout as brazen's `v=1` canonical event stream
//! (NDJSON — one event per line). [`AdapterRunner`] is the exec seam the
//! retry driver ([`super::dispatch`]) depends on; [`SpawnAdapter`] is
//! the production implementation, tests inject a stub.
//!
//! Per-line dispatch is structural: the §4.4 stream emits one event per
//! line, and the harness is the live writer of
//! `<conv-repo>/steps/<conv-id>/<NNN>/response.json` (§3.5). Routing
//! lines through a callback lets the harness append each event to disk
//! as it arrives and stream *content* into the transcript writer's
//! staging sink (§2.3) in the same pass — one stream, two sinks, no
//! read-back.
//!
//! **Exit code is diagnostic (§4.4).** brazen surfaces every failure
//! in-band as an `Error` event on stdout and *also* sets a sysexits
//! exit code computed from the same fact — the event is authoritative,
//! the exit code diagnostic. So a non-zero `bz` exit is NOT a spawn
//! error here: only a failure to *launch* the binary is. brazen dies at
//! once on SIGTERM with no flush (§2.9); the missing trailing `end` on
//! the closed fd is the stop signature, handled by classification, not
//! this runner.
//!
//! **No env forwarding.** Auth and endpoints are entirely brazen's
//! (§4.4): its config resolves via `--config` / `BRAZEN_CONFIG` / XDG,
//! and the harness sets `BRAZEN_CONFIG` only under test isolation — as
//! an inherited process env, never a per-call value the harness
//! threads. The child inherits the harness environment unchanged.

use std::ffi::OsString;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;

/// The provider-adapter binary: brazen's `bz`, resolved on `PATH`
/// unless the global `models.yaml` names an `adapter:` override (ARCH
/// §4.2 / §4.4).
pub const BZ_BIN: &str = "bz";

/// Resolve the adapter binary. One resolution order, most-specific
/// first: the `models.yaml` `adapter:` override (§4.2), else the
/// binding-injected `host` target (`cmd::Fx::adapter_target` — an
/// embedding host naming itself as the adapter, the same injection
/// philosophy as `driver_target`, §3.4), else `bz` on `PATH`. Both named
/// targets are used verbatim, and both skip the load-time version guard
/// in favor of the in-band `MessageStart.v` handshake (§4.4): a named
/// target — config override or host assertion — is identity the caller
/// vouches for, one trust class. The version guard runs only for the
/// default `PATH`-resolved `bz`, when both are `None`.
pub fn resolve_binary(adapter_override: Option<&Path>, host: Option<&Path>) -> OsString {
    match adapter_override.or(host) {
        Some(path) => path.as_os_str().to_os_string(),
        None => OsString::from(BZ_BIN),
    }
}

/// The slice of the adapter contract the harness calls into.
///
/// One subprocess per [`Self::run`]. As the child writes stdout, every
/// completed line (terminator stripped, blanks skipped) is handed to
/// `on_line`. The callback may surface an [`io::Error`] to abort early;
/// otherwise the call returns when the child exits and stdout reaches
/// EOF. A non-zero exit is NOT surfaced — the in-band `Error` event is
/// authoritative and the exit code is diagnostic (§4.4). Only a failure
/// to spawn the binary surfaces as an error.
pub trait AdapterRunner {
    /// Spawn `binary` with `args`, write `stdin_bytes` to its stdin
    /// (closing it after), and route each stdout line through `on_line`.
    fn run(
        &self,
        binary: &OsString,
        args: &[&str],
        stdin_bytes: &[u8],
        on_line: &mut dyn FnMut(&[u8]) -> io::Result<()>,
    ) -> io::Result<()>;
}

/// Default [`AdapterRunner`]. Uses [`Command`] with PATH lookup and
/// inherits the harness environment (test isolation sets `BRAZEN_CONFIG`
/// there, §4.4).
#[derive(Debug, Clone, Copy)]
pub struct SpawnAdapter;

impl AdapterRunner for SpawnAdapter {
    fn run(
        &self,
        binary: &OsString,
        args: &[&str],
        stdin_bytes: &[u8],
        on_line: &mut dyn FnMut(&[u8]) -> io::Result<()>,
    ) -> io::Result<()> {
        let mut child = Command::new(binary)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        // Stdin in a thread so a slow / large request never deadlocks
        // against the child's stdout pipe-buffer fill (the harness
        // tails stdout on the main thread). Writes are best-effort: if
        // `bz` errors before reading stdin, the broken pipe is not a
        // fault — its `Error` event is already on stdout (§4.4).
        let mut stdin = child.stdin.take().expect("stdin is piped");
        let stdin_owned = stdin_bytes.to_vec();
        let stdin_thread = thread::spawn(move || {
            let _ = stdin.write_all(&stdin_owned);
            // Drop closes the fd, signaling EOF.
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

        stdin_thread.join().expect("stdin writer thread panicked");
        // Exit status is diagnostic only (§4.4) — never surfaced.
        let _ = child.wait()?;
        Ok(())
    }
}

/// Run `binary` with `args`, discarding stdin, and return its stdout as
/// one UTF-8 string (lines rejoined by `\n`). Used by the load-time
/// version guard (`bz --version`, §4.4) — the single stdout line `bz`
/// prints is captured through the same exec seam so the guard is
/// stub-testable.
pub fn capture_stdout(
    runner: &dyn AdapterRunner,
    binary: &OsString,
    args: &[&str],
) -> io::Result<String> {
    let mut out: Vec<u8> = Vec::new();
    runner.run(binary, args, b"", &mut |line| {
        if !out.is_empty() {
            out.push(b'\n');
        }
        out.extend_from_slice(line);
        Ok(())
    })?;
    Ok(String::from_utf8_lossy(&out).into_owned())
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

    fn collect_lines<R: AdapterRunner>(
        runner: &R,
        bin: &OsString,
        args: &[&str],
        stdin: &[u8],
    ) -> io::Result<Vec<Vec<u8>>> {
        let mut lines: Vec<Vec<u8>> = Vec::new();
        runner.run(bin, args, stdin, &mut |line| {
            lines.push(line.to_vec());
            Ok(())
        })?;
        Ok(lines)
    }

    #[test]
    fn resolve_binary_defaults_to_bz_on_path() {
        assert_eq!(resolve_binary(None, None), OsString::from("bz"));
    }

    #[test]
    fn resolve_binary_uses_the_adapter_override_verbatim() {
        let over = PathBuf::from("/opt/alt-bz");
        assert_eq!(
            resolve_binary(Some(&over), None),
            OsString::from("/opt/alt-bz")
        );
    }

    #[test]
    fn resolve_binary_uses_the_injected_host_target_verbatim() {
        // No `adapter:` override: the binding-injected host target is used
        // verbatim, above the `bz`-on-PATH default (an embedding host
        // naming itself as the adapter, §3.4).
        let host = PathBuf::from("/opt/host-bz");
        assert_eq!(
            resolve_binary(None, Some(&host)),
            OsString::from("/opt/host-bz")
        );
    }

    #[test]
    fn resolve_binary_prefers_the_override_over_the_injected_host_target() {
        // Both named: the explicit `adapter:` override wins the one
        // resolution order (most-specific first), the host target next.
        let over = PathBuf::from("/opt/alt-bz");
        let host = PathBuf::from("/opt/host-bz");
        assert_eq!(
            resolve_binary(Some(&over), Some(&host)),
            OsString::from("/opt/alt-bz")
        );
    }

    #[test]
    fn spawn_adapter_emits_one_callback_per_stdout_line() {
        let bin = OsString::from("printf");
        let lines = collect_lines(&SpawnAdapter, &bin, &["a\nbb\nccc\n"], b"").unwrap();
        assert_eq!(lines, vec![b"a".to_vec(), b"bb".to_vec(), b"ccc".to_vec()]);
    }

    #[test]
    fn spawn_adapter_handles_crlf_terminators() {
        let bin = OsString::from("printf");
        let lines = collect_lines(&SpawnAdapter, &bin, &["x\r\ny\r\n"], b"").unwrap();
        assert_eq!(lines, vec![b"x".to_vec(), b"y".to_vec()]);
    }

    #[test]
    fn spawn_adapter_pipes_stdin_to_child() {
        let bin = OsString::from("cat");
        let lines = collect_lines(&SpawnAdapter, &bin, &[], b"hello\n").unwrap();
        assert_eq!(lines, vec![b"hello".to_vec()]);
    }

    #[test]
    fn spawn_adapter_reports_spawn_failure() {
        let bin = OsString::from("/no/such/bz-nonesuch");
        let err = collect_lines(&SpawnAdapter, &bin, &[], b"").unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn spawn_adapter_ignores_nonzero_exit() {
        // `false` exits 1 with no stdout; the runner treats the exit as
        // diagnostic (§4.4) and returns Ok with no lines.
        let bin = OsString::from("false");
        let lines = collect_lines(&SpawnAdapter, &bin, &[], b"").unwrap();
        assert!(lines.is_empty());
    }

    #[test]
    fn spawn_adapter_propagates_callback_error() {
        let bin = OsString::from("printf");
        let err = SpawnAdapter
            .run(&bin, &["one\ntwo\n"], b"", &mut |_| {
                Err(io::Error::other("callback bailed"))
            })
            .unwrap_err();
        assert!(err.to_string().contains("callback bailed"));
    }

    #[test]
    fn capture_stdout_rejoins_lines() {
        // Single line: the version-guard shape.
        let bin = OsString::from("printf");
        let out = capture_stdout(&SpawnAdapter, &bin, &["bz 0.0.2\n"]).unwrap();
        assert_eq!(out, "bz 0.0.2");
        // Multiple lines rejoin with `\n` (exercises the separator path).
        let out = capture_stdout(&SpawnAdapter, &bin, &["first\nsecond\n"]).unwrap();
        assert_eq!(out, "first\nsecond");
    }

    #[test]
    fn capture_stdout_surfaces_spawn_failure() {
        let bin = OsString::from("/no/such/bz-nonesuch");
        let err = capture_stdout(&SpawnAdapter, &bin, &["--version"]).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }
}
