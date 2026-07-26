//! The adapter's stderr capture (ARCH §2.3 step record, §4.4).
//!
//! A `bz` that dies before it can speak the in-band contract — a
//! malformed brazen config is the motivating case — writes nothing to
//! stdout and everything to stderr. Without the capture that reads as a
//! bare half-stream (§2.9) and the real complaint is lost.

use super::super::stderr::{TAIL_CHARS as STDERR_TAIL_CHARS, tail as stderr_tail};
use super::*;

/// Drive one model call whose attempts carry the given stderr captures,
/// returning `(result, stderr.log bytes)`.
fn drive_stderr(
    replies: Vec<io::Result<Vec<u8>>>,
    stderrs: Vec<Vec<u8>>,
    max: u32,
) -> (Result<(), Error>, Vec<u8>) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("steps/c/001/response.json");
    let adapter = StubAdapter::with_stderr(replies, stderrs);
    let (result, _, _) = run_with(&path, adapter, retry(max), false);
    let log = std::fs::read(path.with_file_name("stderr.log")).unwrap();
    (result, log)
}

#[test]
fn a_startup_failure_surfaces_its_stderr_and_names_the_artifact() {
    // The Foxglove shape: zero stdout lines (so no terminal `end`) and
    // the real cause on stderr.
    let complaint = b"bz: config /home/u/.config/brazen/config.toml: expected `=`\n";
    let (result, log) = drive_stderr(vec![Ok(Vec::new())], vec![complaint.to_vec()], 3);
    let err = result.unwrap_err();
    let text = err.to_string();
    assert!(matches!(err, Error::AdapterHalfStream { .. }));
    assert!(text.contains("expected `=`"), "{text}");
    assert!(text.contains("stderr.log"), "{text}");
    // The whole capture is the artifact beside `response.json`.
    assert_eq!(log, complaint.to_vec());
}

#[test]
fn every_attempt_appends_its_stderr_and_the_last_one_is_quoted() {
    // A retryable in-band error (attempt 1) then a half-stream (attempt
    // 2): both captures land, and the surfaced tail is the failing
    // attempt's, not the first's.
    let (result, log) = drive_stderr(
        vec![Ok(error_stream(ErrorKind::Transport)), Ok(Vec::new())],
        vec![
            b"first attempt noise\n".to_vec(),
            b"fatal: no auth\n".to_vec(),
        ],
        3,
    );
    let text = result.unwrap_err().to_string();
    assert!(text.contains("fatal: no auth"), "{text}");
    assert!(!text.contains("first attempt noise"), "{text}");
    assert_eq!(log, b"first attempt noise\nfatal: no auth\n".to_vec());
}

#[test]
fn a_clean_run_leaves_an_empty_stderr_artifact() {
    let (result, log) = drive_stderr(vec![Ok(text_stream("hi", FinishReason::Stop))], vec![], 3);
    result.unwrap();
    assert!(log.is_empty(), "an ordinary run says nothing on stderr");
}

#[test]
fn a_silent_half_stream_reads_as_a_genuine_mid_stream_kill() {
    // No stderr at all is itself diagnostic — the §2.9 signature.
    let (result, _) = drive_stderr(vec![Ok(Vec::new())], vec![], 3);
    assert!(result.unwrap_err().to_string().contains("(empty)"));
}

#[test]
fn the_tail_flattens_lines_and_truncates_from_the_front() {
    assert_eq!(stderr_tail(b""), "(empty)");
    assert_eq!(stderr_tail(b"  \n\n"), "(empty)");
    assert_eq!(stderr_tail(b"one\ntwo\n"), "one | two");
    // Longer than the quota: the *tail* survives, marked as clipped.
    let long = "x".repeat(STDERR_TAIL_CHARS) + "-END";
    let tail = stderr_tail(long.as_bytes());
    assert!(tail.starts_with('…'), "{tail}");
    assert!(tail.ends_with("-END"), "{tail}");
    assert_eq!(tail.chars().count(), STDERR_TAIL_CHARS + 1);
}
