//! Tests for the compactor built-in tools (ARCH §2.7).

use super::*;
use crate::template::RealGit;
use std::collections::HashMap;
use std::ffi::OsString;
use std::io::Cursor;
use std::os::unix::ffi::OsStringExt;
use tempfile::TempDir;

/// HashMap-backed stub [`EnvLookup`] — `None` for anything not seeded.
struct StubEnv(HashMap<&'static str, OsString>);
impl EnvLookup for StubEnv {
    fn get(&self, key: &str) -> Option<OsString> {
        self.0.get(key).cloned()
    }
}

fn env(repo: &Path, branch: &str) -> StubEnv {
    let mut m = HashMap::new();
    m.insert(ENV_CONV_REPO, OsString::from(repo));
    m.insert(ENV_CONV_BRANCH, OsString::from(branch));
    StubEnv(m)
}

/// A reader that always errors, for the stdin-read arm.
struct FailingReader;
impl Read for FailingReader {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("stdin boom"))
    }
}

/// A writer that always errors on both `write` and `flush`.
struct FailingWriter;
impl Write for FailingWriter {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("stdout boom"))
    }
    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("flush boom"))
    }
}

/// Init a real git worktree at `<repo>/agents/<branch>` carrying `file`.
fn worktree_repo(repo: &Path, branch: &str, file: &str) -> std::path::PathBuf {
    let wt = workspace::agent_worktree(repo, branch);
    std::fs::create_dir_all(&wt).unwrap();
    let g = RealGit::new();
    g.run(&wt, &["init", "-b", "agents/p1"]).unwrap();
    g.run(&wt, &["config", "user.email", "t@t"]).unwrap();
    g.run(&wt, &["config", "core.hooksPath", "/dev/null"])
        .unwrap();
    g.run(&wt, &["config", "user.name", "t"]).unwrap();
    let f = wt.join(file);
    std::fs::create_dir_all(f.parent().unwrap()).unwrap();
    std::fs::write(&f, "x\n").unwrap();
    g.run(&wt, &["add", "-A"]).unwrap();
    g.run(&wt, &["commit", "-m", "c"]).unwrap();
    wt
}

#[test]
fn write_summary_writes_the_next_summary_file() {
    let repo = TempDir::new().unwrap();
    let wt = workspace::agent_worktree(repo.path(), "p1");
    std::fs::create_dir_all(&wt).unwrap();
    let mut input = Cursor::new(br#"{"content":"digest body\n"}"#.to_vec());
    let mut out = Vec::new();
    run_write_summary(&mut input, &mut out, &env(repo.path(), "p1")).unwrap();
    assert_eq!(
        std::fs::read_to_string(wt.join("summary/001.md")).unwrap(),
        "digest body\n"
    );
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["status"], "written");
    assert_eq!(v["path"], "summary/001.md");
}

#[test]
fn mark_for_deletion_stages_a_removal() {
    let repo = TempDir::new().unwrap();
    worktree_repo(repo.path(), "p1", "messages/001-user.md");
    let mut input = Cursor::new(br#"{"path":"messages/001-user.md"}"#.to_vec());
    let mut out = Vec::new();
    run_mark_for_deletion_with(
        &mut input,
        &mut out,
        &env(repo.path(), "p1"),
        &RealGit::new(),
    )
    .unwrap();
    let wt = workspace::agent_worktree(repo.path(), "p1");
    assert!(!wt.join("messages/001-user.md").exists());
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["status"], "marked");
    assert_eq!(v["path"], "messages/001-user.md");
}

#[test]
fn mark_for_deletion_real_git_wrapper_declines_a_missing_path() {
    // Exercises the `RealGit`-constructing wrapper end to end.
    let repo = TempDir::new().unwrap();
    worktree_repo(repo.path(), "p1", "keep.txt");
    let mut input = Cursor::new(br#"{"path":"no/such.md"}"#.to_vec());
    let mut out = Vec::new();
    let err = run_mark_for_deletion(&mut input, &mut out, &env(repo.path(), "p1")).unwrap_err();
    assert!(matches!(err, Error::Mark(_)), "{err:?}");
}

#[test]
fn invalid_input_json_is_declined() {
    let repo = TempDir::new().unwrap();
    let mut input = Cursor::new(b"{not json".to_vec());
    let mut out = Vec::new();
    let err = run_write_summary(&mut input, &mut out, &env(repo.path(), "p1")).unwrap_err();
    assert!(matches!(err, Error::InvalidJson(_)), "{err:?}");
}

#[test]
fn a_stdin_read_failure_surfaces() {
    let repo = TempDir::new().unwrap();
    let mut out = Vec::new();
    let err = run_write_summary(&mut FailingReader, &mut out, &env(repo.path(), "p1")).unwrap_err();
    assert!(matches!(err, Error::StdinRead(_)), "{err:?}");
}

#[test]
fn missing_repo_env_is_declined() {
    let mut input = Cursor::new(br#"{"content":"x"}"#.to_vec());
    let mut out = Vec::new();
    let err = run_write_summary(&mut input, &mut out, &StubEnv(HashMap::new())).unwrap_err();
    assert!(matches!(err, Error::MissingEnv(ENV_CONV_REPO)), "{err:?}");
}

#[test]
fn missing_branch_env_is_declined() {
    let mut m = HashMap::new();
    m.insert(ENV_CONV_REPO, OsString::from("/x"));
    let mut input = Cursor::new(br#"{"content":"x"}"#.to_vec());
    let mut out = Vec::new();
    let err = run_write_summary(&mut input, &mut out, &StubEnv(m)).unwrap_err();
    assert!(matches!(err, Error::MissingEnv(ENV_CONV_BRANCH)), "{err:?}");
}

#[test]
fn non_utf8_branch_env_is_declined() {
    let mut m = HashMap::new();
    m.insert(ENV_CONV_REPO, OsString::from("/x"));
    m.insert(ENV_CONV_BRANCH, OsString::from_vec(vec![0xff, 0xfe]));
    let mut input = Cursor::new(br#"{"content":"x"}"#.to_vec());
    let mut out = Vec::new();
    let err = run_write_summary(&mut input, &mut out, &StubEnv(m)).unwrap_err();
    assert!(matches!(err, Error::MissingEnv(ENV_CONV_BRANCH)), "{err:?}");
}

#[test]
fn a_write_summary_io_failure_surfaces() {
    // A file where the `agents/` dir should be makes the worktree's
    // `summary/` uncreatable — the fs write fails.
    let repo = TempDir::new().unwrap();
    std::fs::write(repo.path().join("agents"), b"not a dir").unwrap();
    let mut input = Cursor::new(br#"{"content":"x"}"#.to_vec());
    let mut out = Vec::new();
    let err = run_write_summary(&mut input, &mut out, &env(repo.path(), "p1")).unwrap_err();
    assert!(matches!(err, Error::WriteSummary(_)), "{err:?}");
}

#[test]
fn a_stdout_write_failure_surfaces() {
    let repo = TempDir::new().unwrap();
    let wt = workspace::agent_worktree(repo.path(), "p1");
    std::fs::create_dir_all(&wt).unwrap();
    let mut input = Cursor::new(br#"{"content":"x"}"#.to_vec());
    let err =
        run_write_summary(&mut input, &mut FailingWriter, &env(repo.path(), "p1")).unwrap_err();
    assert!(matches!(err, Error::Write(_)), "{err:?}");
    // The stub fails on flush too — a coherent fully-failing writer.
    assert!(FailingWriter.flush().is_err());
}

#[test]
fn error_variants_render_a_message() {
    // The `Display` impls concat into `tool_result.content` on non-zero
    // exit (§3.3) — assert each renders non-empty.
    let e = Error::MissingEnv(ENV_CONV_REPO);
    assert!(!e.to_string().is_empty());
    let e = Error::Mark(crate::prompt::Error::Io(io::Error::other("x")));
    assert!(!e.to_string().is_empty());
    let e = Error::WriteSummary(io::Error::other("x"));
    assert!(!e.to_string().is_empty());
    let e = Error::Write(io::Error::other("x"));
    assert!(!e.to_string().is_empty());
}
