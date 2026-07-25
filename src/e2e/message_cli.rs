//! End-to-end subprocess tests for `lernie message`'s recipient guards:
//! the agent id is one path component (ARCH §2.3) and the recipient
//! exists (§2.11 — "a message is content addressed to an *existing*
//! agent"). Both are declines the real binary must make before it
//! writes anything, so they are pinned through the process boundary
//! rather than only in-process.

use crate::prompt::inbox::{inbox_dir, try_acquire};
use crate::test_support::lernie_binary;
use crate::workspace::fixture;
use std::path::Path;
use std::process::Command;

/// Run `lernie message <ws> <agent> <content>` and hand back
/// `(success, stderr)`.
fn message(ws: &Path, agent: &str, content: &str) -> (bool, String) {
    let out = Command::new(lernie_binary())
        .arg("message")
        .arg(ws)
        .arg(agent)
        .arg(content)
        .output()
        .expect("spawn lernie message");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn an_escaping_agent_id_is_declined_and_writes_nothing_outside_the_workspace() {
    let (holder, ws) = fixture::workspace();
    let (ok, stderr) = message(&ws, "../../victim/pwned", "hello");
    assert!(!ok, "an escaping id must exit non-zero");
    assert!(stderr.contains("lernie message: agent id"), "{stderr}");
    // `<ws>/inbox/../../victim` is `<holder>/victim`.
    assert!(
        !holder.path().join("victim").exists(),
        "nothing is written outside the workspace"
    );
}

#[test]
fn a_recipient_with_no_branch_is_declined_rather_than_silently_deposited() {
    let (_h, ws) = fixture::workspace();
    let (ok, stderr) = message(&ws, "20260101-a1", "hello");
    assert!(!ok, "an unknown recipient must exit non-zero");
    assert!(stderr.contains("existing agent"), "{stderr}");
    assert!(
        !inbox_dir(&ws, "20260101-a1").exists(),
        "the decline creates no inbox directory"
    );
}

#[test]
fn an_existing_recipient_still_receives_its_deposit() {
    let (_h, ws) = fixture::workspace();
    fixture::spawn_root(&ws, "20260101-a1");
    // Hold the executor lease so the post-deposit probe reads Busy and
    // no real driver is spawned into the tempdir (§2.11).
    let _held = try_acquire(&inbox_dir(&ws, "20260101-a1"))
        .unwrap()
        .expect("free lease");
    let (ok, stderr) = message(&ws, "20260101-a1", "hello");
    assert!(ok, "{stderr}");
    let deposits = std::fs::read_dir(inbox_dir(&ws, "20260101-a1"))
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().ends_with(".md"))
        .count();
    assert_eq!(deposits, 1, "exactly one deposit landed");
}
