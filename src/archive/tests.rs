//! Unit tests for [`super`] — `bundle` / `replay` (ARCH §9.2).
//!
//! The git ops are exercised through a recording [`StubGit`]: the pure
//! logic (ref enumeration, primary-id derivation, slice copying, every
//! error arm) is covered here without a real repo. Real-git correctness
//! is locked in end-to-end by `tests/bundle_replay_cli.rs`.

use super::*;
use std::cell::{Cell, RefCell};
use std::io;

/// A `GitRunner` that records `run` invocations and replays a canned
/// `run_capture` output, with injectable failures on either channel.
struct StubGit {
    /// Canned stdout returned by every `run_capture`.
    capture_out: String,
    /// When true, `run_capture` fails.
    fail_capture: bool,
    /// Zero-based `run` index to fail at (`None` = never).
    fail_run_at: Option<usize>,
    runs: RefCell<Vec<Vec<String>>>,
    run_idx: Cell<usize>,
}

impl StubGit {
    fn new(capture_out: &str) -> Self {
        Self {
            capture_out: capture_out.to_owned(),
            fail_capture: false,
            fail_run_at: None,
            runs: RefCell::new(Vec::new()),
            run_idx: Cell::new(0),
        }
    }
    fn fail_capture(mut self) -> Self {
        self.fail_capture = true;
        self
    }
    fn fail_run_at(mut self, idx: usize) -> Self {
        self.fail_run_at = Some(idx);
        self
    }
}

impl GitRunner for StubGit {
    fn run(&self, _dest: &Path, args: &[&str]) -> io::Result<()> {
        let idx = self.run_idx.get();
        self.run_idx.set(idx + 1);
        self.runs
            .borrow_mut()
            .push(args.iter().map(|s| (*s).to_owned()).collect());
        if self.fail_run_at == Some(idx) {
            Err(io::Error::other("stub run fail"))
        } else {
            Ok(())
        }
    }
    fn run_capture(&self, _dest: &Path, _args: &[&str]) -> io::Result<String> {
        if self.fail_capture {
            Err(io::Error::other("stub capture fail"))
        } else {
            Ok(self.capture_out.clone())
        }
    }
}

fn tmp() -> tempfile::TempDir {
    tempfile::TempDir::new().unwrap()
}

fn write(path: &Path, body: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

const REFS: &str = "agents/20260101-p1\nagents/20260101-p1-20260102-c1\n";
const AGENT: &str = "20260101-p1";

// ---- bundle -------------------------------------------------------------

#[test]
fn bundle_writes_bundle_and_matching_slices() {
    let ws = tmp();
    // A matching agent step dir with a file, plus an unrelated sibling.
    write(&ws.path().join("steps/20260101-p1/001/meta.json"), "{}");
    write(&ws.path().join("steps/20260101-other/001/meta.json"), "{}");
    // No inbox dir at all — exercises the missing-slice no-op.
    let out = tmp();
    let git = StubGit::new(REFS);

    bundle(ws.path(), AGENT, out.path(), &git).unwrap();

    // The bundle-create ref list is the enumerated subtree.
    let runs = git.runs.borrow();
    assert_eq!(runs[0][0], "bundle");
    assert_eq!(runs[0][1], "create");
    assert!(runs[0].contains(&"agents/20260101-p1".to_owned()));
    assert!(runs[0].contains(&"agents/20260101-p1-20260102-c1".to_owned()));
    // The matching slice copied; the unrelated sibling did not.
    assert!(out.path().join("steps/20260101-p1/001/meta.json").exists());
    assert!(!out.path().join("steps/20260101-other").exists());
    assert!(!out.path().join("inbox").exists());
}

#[test]
fn bundle_rejects_unknown_agent() {
    let ws = tmp();
    let out = tmp();
    let git = StubGit::new(""); // no branches match
    let err = bundle(ws.path(), AGENT, out.path(), &git).unwrap_err();
    assert!(
        matches!(err, ArchiveError::UnknownAgent(ref a) if a == AGENT),
        "{err:?}"
    );
}

#[test]
fn bundle_surfaces_branch_list_failure() {
    let ws = tmp();
    let out = tmp();
    let git = StubGit::new(REFS).fail_capture();
    let err = bundle(ws.path(), AGENT, out.path(), &git).unwrap_err();
    assert!(
        matches!(
            err,
            ArchiveError::Git {
                op: "branch --list",
                ..
            }
        ),
        "{err:?}"
    );
}

#[test]
fn bundle_surfaces_bundle_create_failure() {
    let ws = tmp();
    let out = tmp();
    let git = StubGit::new(REFS).fail_run_at(0);
    let err = bundle(ws.path(), AGENT, out.path(), &git).unwrap_err();
    assert!(
        matches!(
            err,
            ArchiveError::Git {
                op: "bundle create",
                ..
            }
        ),
        "{err:?}"
    );
}

// ---- replay -------------------------------------------------------------

/// Canned `git bundle list-heads` output for the two-branch subtree,
/// including a short line to exercise the parse filter.
const HEADS: &str =
    "sha1 refs/heads/agents/20260101-p1\nsha2 refs/heads/agents/20260101-p1-20260102-c1\n\n";

/// An archive dir with a (touched) bundle file plus both slices.
fn archive_with_bundle() -> tempfile::TempDir {
    let arch = tmp();
    fs::write(arch.path().join(BUNDLE_FILE), b"").unwrap();
    write(&arch.path().join("steps/20260101-p1/001/meta.json"), "{}");
    write(&arch.path().join("inbox/20260101-p1/user-001.md"), "hi");
    arch
}

#[test]
fn replay_reconstructs_scratch_and_restores_slices() {
    let arch = archive_with_bundle();
    let base = tmp();
    let git = StubGit::new(HEADS);

    let scratch = replay(arch.path(), base.path(), &git).unwrap();

    assert_eq!(scratch, base.path().join(AGENT));
    // init, fetch, worktree add — in order.
    let runs = git.runs.borrow();
    assert_eq!(runs[0][0], "init");
    assert_eq!(runs[1][0], "fetch");
    assert_eq!(runs[2][0], "worktree");
    // Slices restored under the scratch workspace.
    assert!(scratch.join("steps/20260101-p1/001/meta.json").exists());
    assert!(scratch.join("inbox/20260101-p1/user-001.md").exists());
}

#[test]
fn replay_rejects_missing_bundle() {
    let arch = tmp();
    let base = tmp();
    let git = StubGit::new(HEADS);
    let err = replay(arch.path(), base.path(), &git).unwrap_err();
    assert!(matches!(err, ArchiveError::BundleMissing(_)), "{err:?}");
}

#[test]
fn replay_rejects_empty_bundle() {
    let arch = archive_with_bundle();
    let base = tmp();
    let git = StubGit::new(""); // list-heads names nothing
    let err = replay(arch.path(), base.path(), &git).unwrap_err();
    assert!(matches!(err, ArchiveError::EmptyBundle), "{err:?}");
}

#[test]
fn replay_rejects_malformed_bundle() {
    let arch = archive_with_bundle();
    let base = tmp();
    // Two heads sharing no common subtree root.
    let git = StubGit::new("s1 refs/heads/agents/aa-b\ns2 refs/heads/agents/xx-y\n");
    let err = replay(arch.path(), base.path(), &git).unwrap_err();
    assert!(matches!(err, ArchiveError::MalformedBundle(_)), "{err:?}");
}

#[test]
fn replay_rejects_existing_destination() {
    let arch = archive_with_bundle();
    let base = tmp();
    fs::create_dir_all(base.path().join(AGENT)).unwrap();
    let git = StubGit::new(HEADS);
    let err = replay(arch.path(), base.path(), &git).unwrap_err();
    assert!(matches!(err, ArchiveError::DestExists(_)), "{err:?}");
}

#[test]
fn replay_surfaces_list_heads_failure() {
    let arch = archive_with_bundle();
    let base = tmp();
    let git = StubGit::new(HEADS).fail_capture();
    let err = replay(arch.path(), base.path(), &git).unwrap_err();
    assert!(
        matches!(
            err,
            ArchiveError::Git {
                op: "bundle list-heads",
                ..
            }
        ),
        "{err:?}"
    );
}

#[test]
fn replay_surfaces_init_fetch_worktree_failures() {
    for (idx, op) in [(0usize, "init"), (1, "fetch"), (2, "worktree add")] {
        let arch = archive_with_bundle();
        let base = tmp();
        let git = StubGit::new(HEADS).fail_run_at(idx);
        let err = replay(arch.path(), base.path(), &git).unwrap_err();
        match err {
            ArchiveError::Git { op: got, .. } => assert_eq!(got, op),
            other => panic!("expected git {op} failure, got {other:?}"),
        }
    }
}

#[test]
fn replay_tolerates_absent_slices() {
    // Bundle present, but no steps/inbox dirs to restore.
    let arch = tmp();
    fs::write(arch.path().join(BUNDLE_FILE), b"").unwrap();
    let base = tmp();
    let git = StubGit::new(HEADS);
    let scratch = replay(arch.path(), base.path(), &git).unwrap();
    assert!(!scratch.join("steps").exists());
    assert!(!scratch.join("inbox").exists());
}

// ---- error Display ------------------------------------------------------

#[test]
fn error_messages_render() {
    // Exercises every arm's Display so the derived formatting is covered.
    let cases: Vec<ArchiveError> = vec![
        ArchiveError::Io(io::Error::other("x")),
        ArchiveError::Git {
            op: "init",
            source: io::Error::other("x"),
        },
        ArchiveError::UnknownAgent("a".into()),
        ArchiveError::BundleMissing(PathBuf::from("/b")),
        ArchiveError::EmptyBundle,
        ArchiveError::MalformedBundle(vec!["a".into()]),
        ArchiveError::DestExists(PathBuf::from("/d")),
    ];
    for e in cases {
        assert!(!format!("{e}").is_empty());
    }
}
