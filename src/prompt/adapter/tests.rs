//! Tests for the adapter exec seam (ARCH §4.4): binary resolution, the
//! per-line stdout dispatch, and the stderr capture (§2.3).

use super::*;
use std::path::PathBuf;

fn collect_lines<R: AdapterRunner>(
    runner: &R,
    bin: &OsString,
    args: &[&str],
    stdin: &[u8],
) -> io::Result<Vec<Vec<u8>>> {
    Ok(collect(runner, bin, args, stdin)?.0)
}

/// `(stdout lines, captured stderr)` from one run.
fn collect<R: AdapterRunner>(
    runner: &R,
    bin: &OsString,
    args: &[&str],
    stdin: &[u8],
) -> io::Result<(Vec<Vec<u8>>, Vec<u8>)> {
    let mut lines: Vec<Vec<u8>> = Vec::new();
    let stderr = runner.run(bin, args, stdin, &mut |line| {
        lines.push(line.to_vec());
        Ok(())
    })?;
    Ok((lines, stderr))
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
fn spawn_adapter_captures_child_stderr() {
    // The startup-failure shape: nothing on stdout, the real
    // complaint on stderr, non-zero exit (§4.4).
    let bin = OsString::from("sh");
    let (lines, stderr) = collect(
        &SpawnAdapter,
        &bin,
        &["-c", "printf 'config: bad TOML\n' >&2; exit 78"],
        b"",
    )
    .unwrap();
    assert!(lines.is_empty());
    assert_eq!(stderr, b"config: bad TOML\n".to_vec());
}

#[test]
fn spawn_adapter_reads_stderr_concurrently_with_stdout() {
    // 100 KiB of stderr exceeds a pipe buffer: without the reader
    // thread the child would block on stderr while the harness waits
    // on stdout, and neither line below would ever arrive.
    let bin = OsString::from("sh");
    let (lines, stderr) = collect(
        &SpawnAdapter,
        &bin,
        &[
            "-c",
            "head -c 102400 /dev/zero | tr '\\0' 'x' >&2; echo done",
        ],
        b"",
    )
    .unwrap();
    assert_eq!(lines, vec![b"done".to_vec()]);
    assert_eq!(stderr.len(), 102400);
}

#[test]
fn spawn_adapter_captures_no_stderr_on_a_clean_run() {
    let bin = OsString::from("printf");
    let (_, stderr) = collect(&SpawnAdapter, &bin, &["quiet\n"], b"").unwrap();
    assert!(stderr.is_empty());
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
