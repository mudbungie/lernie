//! End-to-end branch-state classification against real conv-repo
//! fixtures.
//!
//! Mirrors the on-disk shape the harness's model-call driver produces —
//! brazen `v=1` NDJSON `response.json` lines under
//! `<conv-repo>/steps/<conv-id>/<NNN>/`. The classifier reads the latest
//! step's `response.json`: absence of a terminal `end` (or a terminal
//! `end` with the writer still holding the fd open, §3.5) is in-flight;
//! a terminal `end` with the fd closed marks the chain stopped. These
//! fixtures write no live writer, so the real `/proc` probe sees the fd
//! as closed.

use super::fixture::Fixture;
use crate::git_tree::{BranchState, GitTree};

#[test]
fn from_repo_in_flight_branch_with_no_response_yet_classifies_as_in_flight() {
    // Branch exists, dispatch commit landed, but no `response.json` has
    // been written for any step. Per ARCH §2.9 (post-amendment):
    // absence of a closed terminal event → InFlight.
    let fx = Fixture::new();
    fx.commit_other("README.md", "initial");
    fx.build_agent("20260427T160000Z-pre0", "no response yet");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.in_flight.len(), 1);
    assert_eq!(tree.in_flight[0].state, BranchState::InFlight);
}

#[test]
fn from_repo_in_flight_branch_with_partial_response_classifies_as_in_flight() {
    // Streaming has begun (text_delta lines on disk) but no terminal
    // event has been emitted yet — the writer is still appending.
    let fx = Fixture::new();
    fx.commit_other("README.md", "initial");
    fx.build_agent("20260427T160100Z-mid0", "mid stream");
    fx.write_response_events(
        "20260427T160100Z-mid0",
        1,
        &[
            r#"{"type":"message_start","v":1,"role":"assistant"}"#,
            r#"{"type":"content_delta","index":0,"delta":{"text_delta":"hello"}}"#,
        ],
    );
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.in_flight[0].state, BranchState::InFlight);
}

#[test]
fn from_repo_in_flight_branch_after_terminal_end_classifies_as_stopped() {
    // The terminal `end` landed and no writer holds the fd; the chain is
    // no longer advancing. Root conversations don't merge back (ARCH
    // §2.3 step 5), so this is the natural terminal state.
    let fx = Fixture::new();
    fx.commit_other("README.md", "initial");
    fx.build_agent("20260427T160200Z-end0", "ended");
    fx.write_response_events(
        "20260427T160200Z-end0",
        1,
        &[
            r#"{"type":"message_start","v":1,"role":"assistant"}"#,
            r#"{"type":"content_delta","index":0,"delta":{"text_delta":"done"}}"#,
            r#"{"type":"finish","reason":"stop"}"#,
            r#"{"type":"end"}"#,
        ],
    );
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.in_flight[0].state, BranchState::Stopped);
}

#[test]
fn from_repo_in_flight_branch_after_error_segment_classifies_as_stopped() {
    // A failed attempt (error + terminal end) with the fd closed marks
    // the chain stopped (§4.4 — a failed step renders as stopped).
    let fx = Fixture::new();
    fx.commit_other("README.md", "initial");
    fx.build_agent("20260427T160300Z-err0", "errored");
    fx.write_response_events(
        "20260427T160300Z-err0",
        1,
        &[
            r#"{"type":"message_start","v":1,"role":"assistant"}"#,
            r#"{"type":"error","kind":"provider","message":"oops"}"#,
            r#"{"type":"end"}"#,
        ],
    );
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.in_flight[0].state, BranchState::Stopped);
}
