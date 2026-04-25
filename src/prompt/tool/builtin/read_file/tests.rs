//! Unit tests for [`super::run`]. Each branch of the function plus
//! every error variant lands in its own test so a coverage regression
//! points at the offending path.

use super::*;
use std::io::Cursor;
use tempfile::{NamedTempFile, TempDir};

fn input_for(path: &std::path::Path) -> Vec<u8> {
    serde_json::json!({ "path": path }).to_string().into_bytes()
}

#[test]
fn happy_path_reads_file_to_stdout() {
    let f = NamedTempFile::new().unwrap();
    std::fs::write(f.path(), b"hello bytes").unwrap();
    let mut stdin = Cursor::new(input_for(f.path()));
    let mut stdout = Vec::new();
    run(&mut stdin, &mut stdout).unwrap();
    assert_eq!(stdout, b"hello bytes");
}

#[test]
fn empty_file_yields_empty_stdout() {
    let f = NamedTempFile::new().unwrap();
    let mut stdin = Cursor::new(input_for(f.path()));
    let mut stdout = Vec::new();
    run(&mut stdin, &mut stdout).unwrap();
    assert!(stdout.is_empty());
}

#[test]
fn invalid_json_input_surfaces_invalid_json() {
    let mut stdin = Cursor::new(b"not json".to_vec());
    let mut stdout = Vec::new();
    let err = run(&mut stdin, &mut stdout).unwrap_err();
    assert!(matches!(err, Error::InvalidJson(_)), "{err}");
}

#[test]
fn missing_path_field_surfaces_invalid_json() {
    let mut stdin = Cursor::new(br#"{"other": "field"}"#.to_vec());
    let mut stdout = Vec::new();
    let err = run(&mut stdin, &mut stdout).unwrap_err();
    assert!(matches!(err, Error::InvalidJson(_)), "{err}");
}

#[test]
fn extra_fields_rejected_by_deny_unknown_fields() {
    // `serde(deny_unknown_fields)` — a typo'd field is not silently
    // ignored, so the model sees an explicit failure.
    let mut stdin = Cursor::new(br#"{"path": "/tmp/x", "extra": 1}"#.to_vec());
    let mut stdout = Vec::new();
    let err = run(&mut stdin, &mut stdout).unwrap_err();
    assert!(matches!(err, Error::InvalidJson(_)), "{err}");
}

#[test]
fn missing_file_surfaces_open_error() {
    let dir = TempDir::new().unwrap();
    let nope = dir.path().join("does-not-exist");
    let mut stdin = Cursor::new(input_for(&nope));
    let mut stdout = Vec::new();
    let err = run(&mut stdin, &mut stdout).unwrap_err();
    let msg = err.to_string();
    assert!(matches!(err, Error::Open { .. }), "{msg}");
    assert!(msg.contains("does-not-exist"), "{msg}");
}

#[test]
fn directory_path_surfaces_read_or_open_error() {
    // `File::open(dir)` succeeds on Linux (the kernel hands back a
    // directory fd), and the subsequent `read_to_end` returns EISDIR,
    // landing on `Error::Read`. On platforms where `File::open`
    // rejects the directory directly, `Error::Open` is the surface.
    // Both are tool-level failures rather than panics, and exercising
    // either lights up the `Error::Read` (or `Error::Open`) branch.
    let dir = TempDir::new().unwrap();
    let mut stdin = Cursor::new(input_for(dir.path()));
    let mut stdout = Vec::new();
    let err = run(&mut stdin, &mut stdout).unwrap_err();
    assert!(
        matches!(err, Error::Open { .. } | Error::Read { .. }),
        "{err}",
    );
}

#[test]
fn oversized_file_surfaces_too_large() {
    // Construct a file just over the cap and confirm the hard-reject
    // path. Use seek-and-write so we don't actually allocate 1 MiB+ of
    // bytes for the test.
    use std::io::{Seek, SeekFrom, Write};
    let f = NamedTempFile::new().unwrap();
    let mut handle = std::fs::OpenOptions::new()
        .write(true)
        .open(f.path())
        .unwrap();
    handle.seek(SeekFrom::Start(MAX_BYTES + 1)).unwrap();
    handle.write_all(&[0u8]).unwrap();
    handle.sync_all().unwrap();
    drop(handle);

    let mut stdin = Cursor::new(input_for(f.path()));
    let mut stdout = Vec::new();
    let err = run(&mut stdin, &mut stdout).unwrap_err();
    let msg = err.to_string();
    assert!(matches!(err, Error::TooLarge { .. }), "{msg}");
    assert!(msg.contains("cap"), "{msg}");
}

#[test]
fn stdin_read_error_surfaces_stdin_read() {
    /// A `Read` impl that always returns an `io::Error` so the
    /// stdin-read branch is exercisable without a closed fd.
    struct BrokenReader;
    impl Read for BrokenReader {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("stdin pipe broken"))
        }
    }
    let mut stdin = BrokenReader;
    let mut stdout = Vec::new();
    let err = run(&mut stdin, &mut stdout).unwrap_err();
    assert!(matches!(err, Error::StdinRead(_)), "{err}");
}

#[test]
fn stdout_write_error_surfaces_write() {
    /// A `Write` that always errors so the final stdout-write branch
    /// is exercisable without an SIGPIPE setup.
    struct BrokenWriter;
    impl Write for BrokenWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("stdout pipe closed"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let f = NamedTempFile::new().unwrap();
    std::fs::write(f.path(), b"non-empty").unwrap();
    let mut stdin = Cursor::new(input_for(f.path()));
    let mut stdout = BrokenWriter;
    let err = run(&mut stdin, &mut stdout).unwrap_err();
    assert!(matches!(err, Error::Write(_)), "{err}");
}
