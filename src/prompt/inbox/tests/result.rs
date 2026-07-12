//! Result-message deposit tests (ARCH §2.6, §2.11): the `epitaph:` /
//! `terminal_ref:` frontmatter, the body-iff-spoke rule, parent
//! derivation, and the root no-op.

use super::super::deposit::{Epitaph, deposit_result};
use super::super::{deposit_child_result, inbox_dir, parent_of};
use crate::prompt::Clock;
use std::path::Path;
use tempfile::TempDir;

struct FixedClock;
impl Clock for FixedClock {
    fn now_iso8601(&self) -> String {
        "2026-07-11T00:00:00Z".into()
    }
    fn now_compact(&self) -> String {
        unreachable!("result deposit never reads the compact clock")
    }
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn epitaph_spellings_are_hyphenated() {
    assert_eq!(Epitaph::FinalResponse.as_str(), "final-response");
    assert_eq!(Epitaph::Stopped.as_str(), "stopped");
    assert_eq!(Epitaph::BudgetExhausted.as_str(), "budget-exhausted");
    assert_eq!(Epitaph::Died.as_str(), "died");
}

#[test]
fn result_message_carries_epitaph_ref_and_body_when_spoke() {
    let ws = TempDir::new().unwrap();
    let path = deposit_result(
        ws.path(),
        "parent",
        "parent-child",
        Epitaph::FinalResponse,
        "abc123",
        Some("all done\n"),
        &FixedClock,
    )
    .unwrap();

    // Deposited into the PARENT's inbox, sender-namespaced by the child.
    assert_eq!(
        path,
        inbox_dir(ws.path(), "parent").join("parent-child-001.md")
    );
    assert_eq!(
        read(&path),
        "---\nfrom: parent-child\ndeposited_at: 2026-07-11T00:00:00Z\n\
         epitaph: final-response\nterminal_ref: abc123\n---\nall done\n"
    );
}

#[test]
fn result_message_omits_body_when_agent_never_spoke() {
    let ws = TempDir::new().unwrap();
    let path = deposit_result(
        ws.path(),
        "parent",
        "parent-child",
        Epitaph::BudgetExhausted,
        "def456",
        None,
        &FixedClock,
    )
    .unwrap();
    // The file ends at the closing frontmatter delimiter — no body.
    assert_eq!(
        read(&path),
        "---\nfrom: parent-child\ndeposited_at: 2026-07-11T00:00:00Z\n\
         epitaph: budget-exhausted\nterminal_ref: def456\n---\n"
    );
}

#[test]
fn parent_of_strips_the_last_descent_segment() {
    // Root: one `<ts>-<short>` segment (two tokens) — no parent.
    assert_eq!(parent_of("20260711T000000Z-a1b2c3d4"), None);
    // Child: parent + one segment.
    assert_eq!(
        parent_of("20260711T000000Z-a1b2c3d4-20260711T000001Z-e5f6a7b8").as_deref(),
        Some("20260711T000000Z-a1b2c3d4")
    );
    // Grandchild strips only the last segment.
    assert_eq!(parent_of("r-aa-c-bb-g-cc").as_deref(), Some("r-aa-c-bb"));
    // Degenerate short ids still obey the two-token rule.
    assert_eq!(parent_of("a-b"), None);
    assert_eq!(parent_of("solo"), None);
}

#[test]
fn deposit_child_result_is_a_noop_for_a_root() {
    let ws = TempDir::new().unwrap();
    let out = deposit_child_result(
        ws.path(),
        "20260711T000000Z-a1b2c3d4",
        Epitaph::FinalResponse,
        "tip",
        Some("hi"),
        &FixedClock,
    )
    .unwrap();
    assert!(out.is_none(), "a root has no parent inbox");
    assert!(!inbox_dir(ws.path(), "20260711T000000Z-a1b2c3d4").exists());
}

#[test]
fn deposit_child_result_deposits_into_parent_for_a_child() {
    let ws = TempDir::new().unwrap();
    let child = "20260711T000000Z-a1b2c3d4-20260711T000001Z-e5f6a7b8";
    let parent = "20260711T000000Z-a1b2c3d4";
    let out = deposit_child_result(
        ws.path(),
        child,
        Epitaph::Stopped,
        "tip9",
        None,
        &FixedClock,
    )
    .unwrap()
    .expect("a child deposits");

    assert_eq!(
        out,
        inbox_dir(ws.path(), parent).join(format!("{child}-001.md"))
    );
    assert!(read(&out).contains("epitaph: stopped"));
    assert!(read(&out).contains("terminal_ref: tip9"));
}
