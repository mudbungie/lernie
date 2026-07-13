//! Inbox-flush tests (ARCH §2.11), error propagation across every seam,
//! and the pure filename matchers.

use super::*;

// ---- flush ----

#[test]
fn flush_launches_only_free_pending_inboxes() {
    let ws = TempDir::new().unwrap();
    // a1: pending + free → launched. a2: held lock → skipped. a3: only a
    // temp stray → no pending, skipped. No agent branches, so no sweep.
    deposit_msg(ws.path(), "a1", "user-001.md");
    deposit_msg(ws.path(), "a2", "user-001.md");
    deposit_msg(ws.path(), "a3", ".user-001.md.tmp");
    let _held = try_acquire(&inbox_dir(ws.path(), "a2"))
        .unwrap()
        .expect("free");
    let git = StubGit::with_branches(&["main"]);
    let launcher = StubLauncher::default();
    let report = scan(ws.path(), &git, &FixedClock, &launcher).unwrap();
    assert_eq!(report.flushed, vec!["a1".to_string()]);
    assert_eq!(launcher.calls(), vec!["a1".to_string()]);
}

#[test]
fn flush_is_a_noop_without_an_inbox_root() {
    let ws = TempDir::new().unwrap();
    let git = StubGit::with_branches(&["main"]);
    let report = scan(ws.path(), &git, &FixedClock, &StubLauncher::default()).unwrap();
    assert_eq!(report, ScanReport::default());
}

// ---- error propagation ----

#[test]
fn branch_enumeration_error_is_surfaced() {
    let ws = TempDir::new().unwrap();
    let git = StubGit::with_branches(&["main", CHILD]).failing("branch");
    let err = scan(ws.path(), &git, &FixedClock, &StubLauncher::default()).unwrap_err();
    assert!(matches!(
        err,
        ScanError::Git {
            op: "branch --list",
            ..
        }
    ));
}

#[test]
fn transcript_read_error_is_surfaced() {
    let ws = TempDir::new().unwrap();
    let git = StubGit::with_branches(&["main", CHILD]).failing("ls-tree");
    let err = scan(ws.path(), &git, &FixedClock, &StubLauncher::default()).unwrap_err();
    assert!(matches!(
        err,
        ScanError::Git {
            op: "ls-tree messages",
            ..
        }
    ));
}

#[test]
fn branch_tip_read_error_is_surfaced() {
    let ws = TempDir::new().unwrap();
    let git = StubGit::with_branches(&["main", CHILD]).failing("rev-parse");
    let err = scan(ws.path(), &git, &FixedClock, &StubLauncher::default()).unwrap_err();
    assert!(matches!(
        err,
        ScanError::Git {
            op: "rev-parse branch tip",
            ..
        }
    ));
}

#[test]
fn probe_error_is_surfaced() {
    let ws = TempDir::new().unwrap();
    // A file where the child's inbox dir should be makes try_acquire fail.
    std::fs::create_dir_all(ws.path().join(INBOX_DIR)).unwrap();
    std::fs::write(inbox_dir(ws.path(), CHILD), b"not a dir").unwrap();
    let git = StubGit::with_branches(&["main", CHILD]);
    let err = scan(ws.path(), &git, &FixedClock, &StubLauncher::default()).unwrap_err();
    assert!(matches!(err, ScanError::Probe { .. }), "{err}");
}

#[test]
fn launch_error_is_surfaced() {
    let ws = TempDir::new().unwrap();
    deposit_msg(ws.path(), "a1", "user-001.md");
    let git = StubGit::with_branches(&["main"]);
    let err = scan(ws.path(), &git, &FixedClock, &FailLauncher).unwrap_err();
    assert!(matches!(err, ScanError::Flush { .. }), "{err}");
}

#[test]
fn inbox_root_read_error_is_surfaced() {
    let ws = TempDir::new().unwrap();
    // `inbox` is a file, not a directory → read_dir errors (not NotFound).
    std::fs::write(ws.path().join(INBOX_DIR), b"not a dir").unwrap();
    let git = StubGit::with_branches(&["main"]);
    let err = scan(ws.path(), &git, &FixedClock, &StubLauncher::default()).unwrap_err();
    assert!(matches!(err, ScanError::InboxRoot { .. }), "{err}");
}

// ---- pure helpers ----

#[test]
fn transcript_line_matcher_rejects_non_matches() {
    assert!(transcript_line_from(
        &format!("messages/007-{CHILD}.md"),
        CHILD
    ));
    assert!(transcript_line_from(&format!("007-{CHILD}.md"), CHILD));
    assert!(!transcript_line_from("messages/007-other.md", CHILD));
    assert!(!transcript_line_from("messages/notes.txt", CHILD));
    assert!(!transcript_line_from("messages/noseq.md", CHILD));
}

#[test]
fn pending_deposit_matcher_rejects_non_deposits() {
    assert!(is_pending_deposit("user-001.md"));
    assert!(is_pending_deposit("a-b-c-042.md"));
    assert!(!is_pending_deposit(".user-001.md.tmp"));
    assert!(!is_pending_deposit("user-abc.md"));
    assert!(!is_pending_deposit("user-001.txt"));
    assert!(!is_pending_deposit("-001.md"));
    assert!(!is_pending_deposit("noseq"));
    assert!(!is_pending_deposit("nohyphen.md"));
}

#[test]
fn cli_run_surfaces_a_scan_error_loudly() {
    // The production wiring runs `git` in `<workspace>/root`, which does
    // not exist here → the enumeration errors → the operator verb
    // propagates it (loud, not best-effort — §2.11 operator framing).
    let ws = TempDir::new().unwrap();
    let err = cli_run(ws.path()).unwrap_err();
    assert!(matches!(err, ScanError::Git { .. }), "{err}");
}

#[test]
fn crash_stranding_is_healed_by_an_explicit_scan() {
    // A hard-crashed child (real branches, no live executor, no result
    // anywhere) strands its parked parent — until an operator runs
    // `lernie scan` (§2.11 "Crashes are a failure class"): the sweep
    // deposits the `died` result on the child's behalf and the flush
    // reports the parent's inbox as launchable. Production wiring
    // (`cli_run`: real git, real clock, the launcher stub).
    let ws = TempDir::new().unwrap();
    let root = ws.path().join(crate::template::ROOT_WORKTREE);
    std::fs::create_dir_all(&root).unwrap();
    let g = crate::template::RealGit::new();
    g.run(&root, &["init", "-b", "main"]).unwrap();
    g.run(&root, &["config", "user.email", "t@test.invalid"])
        .unwrap();
    g.run(&root, &["config", "user.name", "t"]).unwrap();
    g.run(&root, &["commit", "--allow-empty", "-m", "init"])
        .unwrap();
    g.run(&root, &["branch", PARENT]).unwrap();
    g.run(&root, &["branch", CHILD]).unwrap();

    let report = cli_run(ws.path()).unwrap();
    assert_eq!(report.swept, vec![CHILD.to_string()]);
    let deposited = inbox_dir(ws.path(), PARENT).join(format!("{CHILD}-001.md"));
    let body = std::fs::read_to_string(&deposited).unwrap();
    assert!(body.contains("epitaph: died"), "got {body:?}");
    // The flush found the freshly-filled parent inbox launchable; the
    // production launcher is the documented no-op pending `lernie
    // advance`, so the report is the observable.
    assert_eq!(report.flushed, vec![PARENT.to_string()]);
    // The operator-facing summary renders the §8 counts.
    assert_eq!(
        report.to_string(),
        "silent deaths: 1; died deposits swept: 1; drivers launched: 1"
    );
}

#[test]
fn has_pending_reads_a_real_inbox_dir() {
    let ws = TempDir::new().unwrap();
    // Absent dir → no pending (the general path with empty inputs).
    assert!(!has_pending(&ws.path().join("nope")));
    // Present dir with a well-formed deposit → pending.
    deposit_msg(ws.path(), "a1", "user-001.md");
    assert!(has_pending(&inbox_dir(ws.path(), "a1")));
}

#[test]
fn message_from_matcher_is_sender_scoped() {
    assert!(is_message_from(&format!("{CHILD}-003.md"), CHILD));
    assert!(!is_message_from("other-003.md", CHILD));
    assert!(!is_message_from(&format!("{CHILD}-x.md"), CHILD));
}
