//! Unit tests for the step-boundary inbox drain (ARCH §2.11 *Delivery*).
//!
//! `pending` reads a real inbox directory; `recover_strays` and the full
//! `drain` route their `git` verbs through a recording stub, so the
//! on-disk moves and the exact `status`/`add`/`commit` argv are both
//! observable without a live repo.

use super::*;
use crate::template::GitRunner;
use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::path::Path;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;

/// Recording [`GitRunner`] with a scripted `status` reply and an optional
/// failing run index (counted across `run` *and* `run_capture`, the order
/// the drain issues them).
struct RecordGit {
    runs: RefCell<Vec<Vec<String>>>,
    status: String,
    fail_at: Option<usize>,
}

impl RecordGit {
    fn clean() -> Self {
        Self {
            runs: RefCell::new(Vec::new()),
            status: String::new(),
            fail_at: None,
        }
    }
    fn dirty(status: &str) -> Self {
        Self {
            runs: RefCell::new(Vec::new()),
            status: status.to_string(),
            fail_at: None,
        }
    }
    fn failing_at(status: &str, idx: usize) -> Self {
        Self {
            runs: RefCell::new(Vec::new()),
            status: status.to_string(),
            fail_at: Some(idx),
        }
    }
    fn record(&self, args: &[&str]) -> usize {
        let mut r = self.runs.borrow_mut();
        let idx = r.len();
        r.push(args.iter().map(|s| (*s).to_string()).collect());
        idx
    }
}

impl GitRunner for RecordGit {
    fn run(&self, _dest: &Path, args: &[&str]) -> io::Result<()> {
        let idx = self.record(args);
        if self.fail_at == Some(idx) {
            Err(io::Error::other("boom"))
        } else {
            Ok(())
        }
    }
    fn run_capture(&self, _dest: &Path, args: &[&str]) -> io::Result<String> {
        let idx = self.record(args);
        if self.fail_at == Some(idx) {
            Err(io::Error::other("status boom"))
        } else {
            Ok(self.status.clone())
        }
    }
}

/// Write a deposit file into `inbox` with the given name, body, and mtime
/// so ordering is deterministic (the OS clock's resolution is not relied
/// on).
fn deposit_file(inbox: &Path, name: &str, body: &str, mtime: SystemTime) {
    std::fs::create_dir_all(inbox).unwrap();
    let path = inbox.join(name);
    std::fs::write(&path, body).unwrap();
    File::open(&path).unwrap().set_modified(mtime).unwrap();
}

fn at(secs: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(secs)
}

#[test]
fn pending_is_empty_when_inbox_is_absent() {
    let dir = TempDir::new().unwrap();
    let got = pending(&dir.path().join("inbox/nobody")).unwrap();
    assert!(got.is_empty());
}

#[test]
fn pending_surfaces_a_non_not_found_read_error() {
    // A regular file where the inbox dir is expected: read_dir fails with
    // a non-NotFound error, which surfaces rather than reading as empty.
    let dir = TempDir::new().unwrap();
    let not_a_dir = dir.path().join("inbox-file");
    std::fs::write(&not_a_dir, b"x").unwrap();
    let err = pending(&not_a_dir).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn pending_sorts_by_mtime_then_filename_and_skips_non_deposits() {
    let dir = TempDir::new().unwrap();
    let inbox = dir.path().join("inbox/a");
    // Later mtime, earliest-sorting name — sorts last (mtime dominates).
    deposit_file(&inbox, "alice-001.md", "a", at(30));
    // Same mtime pair, filename breaks the tie: bob before zed.
    deposit_file(&inbox, "zed-9-002.md", "z", at(10));
    deposit_file(&inbox, "bob-001.md", "b", at(10));
    // A hyphenated agent id keeps every hyphen but the numeric tail.
    deposit_file(&inbox, "a-b-c-004.md", "c", at(20));
    // Non-deposits: wrong extension, non-numeric tail, empty sender, and
    // an in-flight temp file — none contribute.
    deposit_file(&inbox, "notes.txt", "n", at(1));
    deposit_file(&inbox, "nope-x.md", "x", at(1));
    deposit_file(&inbox, "-001.md", "e", at(1));
    deposit_file(&inbox, ".bob-003.md.tmp", "t", at(1));

    let got = pending(&inbox).unwrap();
    let order: Vec<(&str, &str)> = got
        .iter()
        .map(|p| (p.name.as_str(), p.sender.as_str()))
        .collect();
    assert_eq!(
        order,
        vec![
            ("bob-001.md", "bob"),
            ("zed-9-002.md", "zed-9"),
            ("a-b-c-004.md", "a-b-c"),
            ("alice-001.md", "alice"),
        ]
    );
}

#[test]
fn recover_strays_is_a_noop_on_a_clean_messages_dir() {
    let dir = TempDir::new().unwrap();
    let git = RecordGit::clean();
    recover_strays(dir.path(), "conv-1", &git).unwrap();
    // Only the status probe ran — nothing to recover.
    let runs = git.runs.borrow();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0], vec!["status", "--porcelain", "--", "messages"]);
}

#[test]
fn recover_strays_commits_an_uncommitted_stray() {
    let dir = TempDir::new().unwrap();
    let git = RecordGit::dirty("?? messages/003-user.md\n");
    recover_strays(dir.path(), "conv-9", &git).unwrap();
    let runs = git.runs.borrow();
    assert_eq!(runs.len(), 3);
    assert_eq!(runs[1], vec!["add", "messages"]);
    assert_eq!(runs[2][0], "commit");
    assert!(runs[2][2].contains("recover delivered stray"));
    assert!(runs[2][2].contains("[conv-9]"));
}

#[test]
fn recover_strays_surfaces_status_add_and_commit_errors() {
    let dir = TempDir::new().unwrap();
    // status (run 0) fails.
    let err = recover_strays(dir.path(), "c", &RecordGit::failing_at("", 0)).unwrap_err();
    assert!(
        matches!(
            err,
            Error::Git {
                op: "drain status",
                ..
            }
        ),
        "got {err:?}"
    );
    // add (run 1) fails.
    let err =
        recover_strays(dir.path(), "c", &RecordGit::failing_at("?? messages/x", 1)).unwrap_err();
    assert!(
        matches!(
            err,
            Error::Git {
                op: "drain recover add",
                ..
            }
        ),
        "got {err:?}"
    );
    // commit (run 2) fails.
    let err =
        recover_strays(dir.path(), "c", &RecordGit::failing_at("?? messages/x", 2)).unwrap_err();
    assert!(
        matches!(
            err,
            Error::Git {
                op: "drain recover commit",
                ..
            }
        ),
        "got {err:?}"
    );
}

#[test]
fn drain_delivers_pending_messages_in_mtime_order_committing_each() {
    let dir = TempDir::new().unwrap();
    let worktree = dir.path().join("wt");
    std::fs::create_dir_all(&worktree).unwrap();
    let inbox = dir.path().join("inbox/agent");
    // bob (earlier) then alice (later): committed sequence follows mtime.
    deposit_file(&inbox, "alice-001.md", "---\nfrom: alice\n---\nhi", at(20));
    deposit_file(&inbox, "bob-001.md", "---\nfrom: bob\n---\nyo", at(10));

    let git = RecordGit::clean();
    drain(&worktree, &inbox, "conv-7", &git).unwrap();

    // Both files moved out of the inbox into the transcript, frontmatter
    // and body intact (the rename copies bytes untouched, §2.11).
    assert!(!inbox.join("bob-001.md").exists());
    assert!(!inbox.join("alice-001.md").exists());
    let first = std::fs::read_to_string(worktree.join("messages/001-bob.md")).unwrap();
    assert_eq!(first, "---\nfrom: bob\n---\nyo");
    let second = std::fs::read_to_string(worktree.join("messages/002-alice.md")).unwrap();
    assert_eq!(second, "---\nfrom: alice\n---\nhi");

    // Git op log: status probe, then per-message add+commit at 001, 002.
    let runs = git.runs.borrow();
    assert_eq!(runs[0], vec!["status", "--porcelain", "--", "messages"]);
    assert_eq!(runs[1], vec!["add", "messages/001-bob.md"]);
    assert!(runs[2][2].contains("transcript 001: bob"));
    assert_eq!(runs[3], vec!["add", "messages/002-alice.md"]);
    assert!(runs[4][2].contains("transcript 002: alice"));
}

#[test]
fn drain_applies_the_work_product_transfer_for_a_result_message() {
    // A deposited message carrying `terminal_ref:` is a result message
    // (§2.6): the drain runs the work-product transfer (merge-base + diff)
    // before its delivery commit. With the stub git the diff writes no
    // patch, so the transfer is an empty no-op commit — but the transfer
    // path is exercised, and the message still delivers.
    let dir = TempDir::new().unwrap();
    let worktree = dir.path().join("wt");
    std::fs::create_dir_all(&worktree).unwrap();
    let inbox = dir.path().join("inbox/parent");
    deposit_file(
        &inbox,
        "parent-kid-001.md",
        "---\nfrom: parent-kid\nepitaph: final-response\nterminal_ref: abc123\n---\ndone",
        at(10),
    );

    let git = RecordGit::clean();
    drain(&worktree, &inbox, "parent", &git).unwrap();

    // The transfer ran (merge-base against the terminal ref, then the
    // filtered diff) ahead of the delivery add+commit.
    let runs = git.runs.borrow();
    assert_eq!(runs[0], vec!["status", "--porcelain", "--", "messages"]);
    assert_eq!(runs[1][..2], ["merge-base", "parent"]);
    assert_eq!(runs[1][2], "abc123");
    assert_eq!(runs[2][0], "diff");
    assert_eq!(runs[3], vec!["add", "messages/001-parent-kid.md"]);
    assert!(runs[4][2].contains("transcript 001: parent-kid"));
    // The message was delivered out of the inbox.
    assert!(!inbox.join("parent-kid-001.md").exists());
    assert!(worktree.join("messages/001-parent-kid.md").exists());
}

#[test]
fn drain_on_an_absent_inbox_only_probes_for_strays() {
    let dir = TempDir::new().unwrap();
    let worktree = dir.path().join("wt");
    std::fs::create_dir_all(&worktree).unwrap();
    let git = RecordGit::clean();
    drain(&worktree, &dir.path().join("inbox/nobody"), "c", &git).unwrap();
    // Clean stray probe, no inbox files: exactly the one status run.
    assert_eq!(git.runs.borrow().len(), 1);
}
