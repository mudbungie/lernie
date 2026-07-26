//! Silent-death sweep tests (ARCH §8): the never-deposited-child deposit,
//! idempotence, the returned/driven/root exclusions, and died-mid-work.

use super::*;

// ---- the never-deposited child gets one died deposit ----

#[test]
fn sweep_deposits_died_for_a_child_that_never_returned() {
    let ws = TempDir::new().unwrap();
    let git = StubGit::with_branches(&["main", PARENT, CHILD]).tip(CHILD, "cafef00d");
    let launcher = StubLauncher::default();
    let report = scan(ws.path(), &git, &FixedClock, &launcher).unwrap();

    assert_eq!(report.swept, vec![CHILD.to_string()]);
    assert_eq!(report.silent_deaths, 1);
    // The deposit is a message *from the child* in the parent's inbox.
    let deposited = inbox_dir(ws.path(), PARENT).join(format!("{CHILD}-001.md"));
    let body = std::fs::read_to_string(&deposited).unwrap();
    assert!(
        body.contains(&format!("from: {CHILD}")),
        "sender is the child"
    );
    assert!(body.contains("epitaph: died"));
    assert!(body.contains("terminal_ref: cafef00d"));
    // The flush that follows launches a driver for the freshly-filled
    // parent inbox — never for the child (nothing pending there).
    assert_eq!(report.flushed, vec![PARENT.to_string()]);
    assert_eq!(launcher.invocations(), vec![PARENT.to_string()]);
}

#[test]
fn sweep_is_idempotent_across_a_double_scan() {
    let ws = TempDir::new().unwrap();
    let git = StubGit::with_branches(&["main", PARENT, CHILD]);
    let launcher = StubLauncher::default();

    let first = scan(ws.path(), &git, &FixedClock, &launcher).unwrap();
    assert_eq!(first.swept, vec![CHILD.to_string()]);

    // Second scan: the prior sweep's own deposit is a message from the
    // child in the parent's inbox, so the never-deposited derivation is
    // now false — no re-deposit, and exactly one died file remains.
    let second = scan(ws.path(), &git, &FixedClock, &launcher).unwrap();
    assert!(second.swept.is_empty(), "no re-deposit on re-scan");
    assert_eq!(second.silent_deaths, 0);
    let count = std::fs::read_dir(inbox_dir(ws.path(), PARENT))
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with(CHILD))
        .count();
    assert_eq!(count, 1, "exactly one died deposit");
}

// ---- exclusions: already-returned, driven, root ----

#[test]
fn child_with_a_delivered_transcript_message_is_not_swept() {
    let ws = TempDir::new().unwrap();
    // The parent branch already carries a delivered return from the child.
    let git = StubGit::with_branches(&["main", PARENT, CHILD])
        .ls_tree(PARENT, &format!("messages/003-{CHILD}.md\n"));
    let report = scan(ws.path(), &git, &FixedClock, &StubLauncher::default()).unwrap();
    assert!(
        report.swept.is_empty(),
        "delivered return counts as returned"
    );
    assert_eq!(report.silent_deaths, 0);
}

#[test]
fn child_with_an_undelivered_inbox_message_is_not_swept() {
    let ws = TempDir::new().unwrap();
    // A result already sits undelivered in the parent's inbox.
    deposit_msg(ws.path(), PARENT, &format!("{CHILD}-001.md"));
    let git = StubGit::with_branches(&["main", PARENT, CHILD]);
    let report = scan(ws.path(), &git, &FixedClock, &StubLauncher::default()).unwrap();
    assert!(report.swept.is_empty());
    assert_eq!(report.silent_deaths, 0);
}

#[test]
fn a_driven_child_is_never_swept() {
    let ws = TempDir::new().unwrap();
    // Simulate a live executor by holding the child's lock across the scan.
    let _held = try_acquire(&inbox_dir(ws.path(), CHILD))
        .unwrap()
        .expect("free");
    let git = StubGit::with_branches(&["main", PARENT, CHILD]);
    let report = scan(ws.path(), &git, &FixedClock, &StubLauncher::default()).unwrap();
    assert!(report.swept.is_empty(), "held lock is not a silent death");
    assert_eq!(report.silent_deaths, 0);
}

#[test]
fn a_root_is_counted_but_never_deposited() {
    let ws = TempDir::new().unwrap();
    // A root that died mid-work: latest response.json has no terminal end.
    write_response(ws.path(), PARENT, "001", KILLED);
    let git = StubGit::with_branches(&["main", PARENT]);
    let report = scan(ws.path(), &git, &FixedClock, &StubLauncher::default()).unwrap();
    assert!(report.swept.is_empty(), "a root has no parent inbox");
    assert_eq!(
        report.silent_deaths, 1,
        "died-mid-work root is a silent death"
    );
}

// ---- died_mid_work derivation over steps/ ----

#[test]
fn died_mid_work_reads_the_latest_step_response() {
    let ws = TempDir::new().unwrap();
    // Step 1 completed cleanly; step 2 was killed — the latest governs.
    write_response(ws.path(), PARENT, "001", COMPLETE);
    write_response(ws.path(), PARENT, "002", KILLED);
    assert!(died_mid_work(ws.path(), PARENT));
}

#[test]
fn a_complete_latest_response_is_not_a_death() {
    let ws = TempDir::new().unwrap();
    write_response(ws.path(), PARENT, "001", KILLED);
    write_response(ws.path(), PARENT, "002", COMPLETE);
    assert!(!died_mid_work(ws.path(), PARENT));
}

#[test]
fn no_steps_tree_is_not_a_death() {
    let ws = TempDir::new().unwrap();
    assert!(
        !died_mid_work(ws.path(), CHILD),
        "shipped child shape: no steps/"
    );
    // A steps/ dir with no numeric step is likewise silent.
    std::fs::create_dir_all(ws.path().join(STEPS_DIR).join(CHILD).join("junk")).unwrap();
    assert!(!died_mid_work(ws.path(), CHILD));
    // A numeric latest step whose response.json is absent (a death before
    // the model call even opened one) reads silent — no framing to judge.
    std::fs::create_dir_all(ws.path().join(STEPS_DIR).join(CHILD).join("001")).unwrap();
    assert!(!died_mid_work(ws.path(), CHILD));
}
