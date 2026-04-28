//! End-to-end branch-state classification (bl-de6b) against real
//! conv-repo fixtures.
//!
//! Mirrors the on-disk shape `src/prompt/dispatch::stream::run_complete`
//! produces — JSONL `response.json` lines under
//! `<conv-repo>/steps/<conv-id>/<NNN>/`. Per ARCH §2.9 (post-bl-de6b
//! amendment) the classifier reads the latest step's `response.json`
//! and treats the absence of a §4.4 terminal event as in-flight; a
//! `message_stop` or `error` line marks the chain as no longer
//! advancing.

use super::fixture::Fixture;
use crate::git_tree::{BranchState, GitTree};

#[test]
fn from_repo_in_flight_branch_with_no_response_yet_classifies_as_in_flight() {
    // Branch exists, dispatch commit landed, but no `response.json` has
    // been written for any step. Per ARCH §2.9 (post-amendment):
    // absence of a closed terminal event → InFlight.
    let fx = Fixture::new();
    fx.commit_other("README.md", "initial");
    fx.build_v03_in_flight("20260427T160000Z-pre0", "no response yet");
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
    fx.build_v03_in_flight("20260427T160100Z-mid0", "mid stream");
    fx.write_response_events(
        "20260427T160100Z-mid0",
        1,
        &[
            r#"{"type":"message_start","message":{"id":"m1"}}"#,
            r#"{"type":"text_delta","index":0,"text":"hello"}"#,
        ],
    );
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.in_flight[0].state, BranchState::InFlight);
}

#[test]
fn from_repo_in_flight_branch_after_message_stop_classifies_as_stopped() {
    // The terminal `message_stop` event has landed; the chain is no
    // longer advancing. Root conversations don't merge back (ARCH §2.3
    // step 5), so this is the natural terminal state for a root
    // conversation.
    let fx = Fixture::new();
    fx.commit_other("README.md", "initial");
    fx.build_v03_in_flight("20260427T160200Z-end0", "ended");
    fx.write_response_events(
        "20260427T160200Z-end0",
        1,
        &[
            r#"{"type":"message_start","message":{"id":"m1"}}"#,
            r#"{"type":"text_delta","index":0,"text":"done"}"#,
            r#"{"type":"message_stop","usage":{"input_tokens":1,"output_tokens":1},"api_calls":1}"#,
        ],
    );
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.in_flight[0].state, BranchState::Stopped);
}

#[test]
fn from_repo_in_flight_branch_after_error_event_classifies_as_stopped() {
    // The terminal `error` event landed instead of `message_stop`.
    // Per §4.4 these are both terminal; the classifier treats them the
    // same (the chain is no longer advancing).
    let fx = Fixture::new();
    fx.commit_other("README.md", "initial");
    fx.build_v03_in_flight("20260427T160300Z-err0", "errored");
    fx.write_response_events(
        "20260427T160300Z-err0",
        1,
        &[
            r#"{"type":"message_start","message":{"id":"m1"}}"#,
            r#"{"type":"error","kind":"fatal","http_status":500,"message":"oops"}"#,
        ],
    );
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.in_flight[0].state, BranchState::Stopped);
}
