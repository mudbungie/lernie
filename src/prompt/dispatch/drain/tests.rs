//! Unit tests for the step-boundary inbox drain (ARCH §2.11 *Delivery*).
//!
//! `pending` reads a real inbox directory; `recover_strays` and `drain`
//! route their `git` verbs through a recording stub, so the on-disk moves
//! and the exact argv are observable without a live repo.

use super::*;
use crate::template::GitRunner;
use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::path::Path;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;

/// Recording [`GitRunner`]: a scripted `status` reply and an optional
/// failing run index (counted across `run` and `run_capture` alike).
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
/// so ordering is deterministic (never the OS clock's resolution).
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
    // Later mtime, earliest name — sorts last (mtime dominates).
    deposit_file(&inbox, "alice-001.md", "a", at(30));
    // Same mtime, filename breaks the tie: bob before zed.
    deposit_file(&inbox, "zed-9-002.md", "z", at(10));
    deposit_file(&inbox, "bob-001.md", "b", at(10));
    // A hyphenated id keeps every hyphen but the numeric tail.
    deposit_file(&inbox, "a-b-c-004.md", "c", at(20));
    // Non-deposits contribute nothing: bad extension, bad tail, no sender.
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
    // The three ops fail in turn: status (run 0), add (1), commit (2).
    let dir = TempDir::new().unwrap();
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

    // Both moved into the transcript, bytes untouched by the rename.
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

/// A result message's on-disk body (§2.6) from `sender`.
fn result_body(sender: &str) -> String {
    format!("---\nfrom: {sender}\nepitaph: final-response\nterminal_ref: abc123\n---\ndone")
}

#[test]
fn drain_leaves_an_own_childs_result_message_in_the_inbox_for_the_interpreter() {
    // A deposit carrying `terminal_ref:` *from this agent's own child* is
    // a result message (§2.6): a §6 lifecycle circumstance, not steering.
    let dir = TempDir::new().unwrap();
    let worktree = dir.path().join("wt");
    std::fs::create_dir_all(&worktree).unwrap();
    let inbox = dir.path().join("inbox/20260101-a1");
    let child = "20260101-a1-20260102-b2";
    deposit_file(
        &inbox,
        &format!("{child}-001.md"),
        &result_body(child),
        at(10),
    );

    let git = RecordGit::clean();
    drain(&worktree, &inbox, "20260101-a1", &git).unwrap();

    // No delivery: only the clean stray probe ran; the file stays put.
    let runs = git.runs.borrow();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0], vec!["status", "--porcelain", "--", "messages"]);
    assert!(inbox.join(format!("{child}-001.md")).exists());
    assert!(!worktree.join(format!("messages/001-{child}.md")).exists());
}

#[test]
fn drain_delivers_a_reply_from_an_agent_this_one_never_dispatched() {
    // §2.6: the return — the transfer and the §6 bindings — is the
    // *dispatcher's* business. A sibling's reply carries the same
    // frontmatter but neither relationship, so it just delivers (§2.11).
    let dir = TempDir::new().unwrap();
    let worktree = dir.path().join("wt");
    std::fs::create_dir_all(&worktree).unwrap();
    let inbox = dir.path().join("inbox/20260101-a1-20260102-b2");
    let sibling = "20260101-a1-20260102-c3";
    deposit_file(
        &inbox,
        &format!("{sibling}-001.md"),
        &result_body(sibling),
        at(10),
    );

    let git = RecordGit::clean();
    let delivery = drain(&worktree, &inbox, "20260101-a1-20260102-b2", &git).unwrap();

    assert_eq!(delivery.delivered, 1);
    assert!(delivery.left.is_empty(), "nothing is held for a §6 binding");
    assert!(!inbox.join(format!("{sibling}-001.md")).exists());
    let landed = std::fs::read_to_string(worktree.join(format!("messages/001-{sibling}.md")));
    assert!(landed.unwrap().contains("epitaph: final-response"));
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
