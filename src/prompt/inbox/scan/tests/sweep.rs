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
    assert_eq!(report.silent_deaths, vec![CHILD.to_string()]);
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
    assert!(second.silent_deaths.is_empty());
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
    assert!(report.silent_deaths.is_empty());
}

#[test]
fn child_with_an_undelivered_inbox_message_is_not_swept() {
    let ws = TempDir::new().unwrap();
    // A result already sits undelivered in the parent's inbox.
    deposit_msg(ws.path(), PARENT, &format!("{CHILD}-001.md"));
    let git = StubGit::with_branches(&["main", PARENT, CHILD]);
    let report = scan(ws.path(), &git, &FixedClock, &StubLauncher::default()).unwrap();
    assert!(report.swept.is_empty());
    assert!(report.silent_deaths.is_empty());
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
    assert!(report.silent_deaths.is_empty());
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
        report.silent_deaths,
        vec![PARENT.to_string()],
        "died-mid-work root is a silent death, surfaced by name"
    );
}

#[test]
fn a_root_with_a_failed_model_call_is_a_named_silent_death() {
    // bl-ee80: a non-retryable (or retries-exhausted) model-call error
    // ends its segment cleanly — an `Error` then the terminal `end` — so
    // the branch looks idle to a no-terminal-`end` test while being
    // permanently unable to advance (§2.10: the model call never settled
    // complete, no transcript entry committed). The sweep classifies it
    // dead from the same framing tail and, the root having no parent
    // inbox to deposit into, its *name* in the report is the surfacing.
    let ws = TempDir::new().unwrap();
    write_response(ws.path(), PARENT, "001", FAILED);
    let git = StubGit::with_branches(&["main", PARENT]);
    let report = scan(ws.path(), &git, &FixedClock, &StubLauncher::default()).unwrap();
    assert!(report.swept.is_empty(), "a root gets no deposit");
    assert_eq!(report.silent_deaths, vec![PARENT.to_string()]);
    assert!(
        report
            .to_string()
            .contains(&format!("silent deaths: 1 ({PARENT})")),
        "{report}"
    );
}

#[test]
fn a_branch_whose_derived_parent_has_no_ref_is_treated_as_a_root() {
    // bl-025b. `parent_of` is string arithmetic over the hyphenated
    // descent; whether that address names an agent is a query against
    // the `agents/*` registry. An odd-token branch (or one whose parent
    // ref was deleted) derives an address that holds no ref. Before the
    // intersection the sweep asked git for `agents/<that>` anyway and
    // the 128 aborted the WHOLE pass — sweep and flush both — so mail
    // pending elsewhere went unflushed. It is nobody's child: no parent
    // inbox, so nothing to deposit and nothing to ask.
    let ws = TempDir::new().unwrap();
    let orphan = format!("{PARENT}-c0ffee");
    // The one-token suffix makes three tokens; `parent_of` strips two.
    assert_eq!(parent_of(&orphan).as_deref(), Some("20260101"));
    deposit_msg(ws.path(), PARENT, "user-001.md");
    let git = StubGit::with_branches(&[PARENT, orphan.as_str()]);
    let launcher = StubLauncher::default();
    let report = scan(ws.path(), &git, &FixedClock, &launcher).unwrap();

    assert!(report.swept.is_empty(), "no parent inbox to deposit into");
    assert!(
        report.silent_deaths.is_empty(),
        "alive-and-quiet, like a root"
    );
    assert!(
        !git.asked_ls_tree_for("20260101"),
        "no git question about a ref the registry does not hold"
    );
    // The pass survives to its second half: the flush still runs.
    assert_eq!(report.flushed, vec![PARENT.to_string()]);
    assert_eq!(launcher.invocations(), vec![PARENT.to_string()]);
}

#[test]
fn an_absent_parent_inbox_reads_as_no_undelivered_return() {
    // `returned`'s inbox half is total over a missing directory — the
    // general path with empty inputs, not a bootstrap case. Asserted at
    // the predicate's own level because the sweep cannot reach it: the
    // registry intersection (bl-025b) means the parent is always an
    // enumerated agent, a refname sorts before every id extending it, and
    // that earlier iteration's lock probe `create_dir_all`s the inbox. An
    // *unreadable* one still lands here in production, so the arm stays.
    let ws = TempDir::new().unwrap();
    assert!(!inbox_dir(ws.path(), PARENT).exists());
    assert!(!returned(ws.path(), &StubGit::default(), PARENT, CHILD).unwrap());
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
fn a_failed_latest_response_is_a_death() {
    // §2.10: an `Error`-terminated final segment (with its clean `end`)
    // is equally dead — the model call never settled complete.
    let ws = TempDir::new().unwrap();
    write_response(ws.path(), PARENT, "001", FAILED);
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

// ---- the durable returned mark (bl-2c06) ----

#[test]
fn a_marked_child_is_not_swept_after_its_message_was_consumed() {
    // The bl-2c06 defect, pinned: a compactor returned final-response,
    // its deposit was interpreted (land_compaction) and the message file
    // consumed with no transcript entry — the on-disk state here (no
    // inbox file, no messages/NNN-<child>.md) is exactly what the sweep
    // used to misread as a silent death, depositing a false `died`
    // epitaph into the parent. The durable mark the deposit wrote is the
    // evidence that survives the consumption.
    let ws = TempDir::new().unwrap();
    let git = StubGit::with_branches(&["main", PARENT, CHILD]).marked(CHILD, "cafef00d");
    let launcher = StubLauncher::default();
    let report = scan(ws.path(), &git, &FixedClock, &launcher).unwrap();

    assert!(report.swept.is_empty(), "a marked return is never re-swept");
    assert!(report.silent_deaths.is_empty());
    assert!(
        !inbox_dir(ws.path(), PARENT)
            .join(format!("{CHILD}-001.md"))
            .exists(),
        "no false died deposit"
    );
    assert!(launcher.invocations().is_empty());
}

#[test]
fn the_sweeps_own_deposit_writes_the_mark_it_later_reads() {
    // Idempotence now rides the mark as well as the file: the sweep's
    // died deposit goes through the one deposit_result seam, so even if
    // the deposited file is later delivered and compacted away, a
    // re-scan still sees the return.
    let ws = TempDir::new().unwrap();
    let git = StubGit::with_branches(&["main", PARENT, CHILD]).tip(CHILD, "cafef00d");
    let launcher = StubLauncher::default();
    let first = scan(ws.path(), &git, &FixedClock, &launcher).unwrap();
    assert_eq!(first.swept, vec![CHILD.to_string()]);

    // Erase every message-level trace, as delivery-then-compaction would.
    std::fs::remove_file(inbox_dir(ws.path(), PARENT).join(format!("{CHILD}-001.md"))).unwrap();
    let second = scan(ws.path(), &git, &FixedClock, &launcher).unwrap();
    assert!(
        second.swept.is_empty(),
        "the mark alone blocks a re-deposit"
    );
    assert!(second.silent_deaths.is_empty());
}

#[test]
fn returned_reads_the_mark_ahead_of_inbox_and_transcript() {
    let ws = TempDir::new().unwrap();
    let marked = StubGit::default().marked(CHILD, "cafef00d");
    assert!(returned(ws.path(), &marked, PARENT, CHILD).unwrap());
    let unmarked = StubGit::default();
    assert!(!returned(ws.path(), &unmarked, PARENT, CHILD).unwrap());
}
