//! The name fact, end to end against a real workspace: settled by the
//! dispatch commit, read back out of the ref, unique at creation, and
//! resolved id-or-unique-name with ambiguity refused.

use super::{Unavailable, is_id_timestamp, named, read, require_available, resolve, settle};
use crate::template::{GitRunner, RealGit};
use crate::workspace::fixture;
use std::path::Path;

/// Two well-formed ids in the real grammar (`<ts>-<short>`, §2.3).
const ID_A: &str = "20260101T000000Z-aaaaaaaa";
const ID_B: &str = "20260102T000000Z-bbbbbbbb";

/// Fork a root agent and land the name fact on a commit of its own —
/// the shape [`settle`] produces inside a dispatch commit.
fn agent(ws: &Path, id: &str, name: Option<&str>) {
    let git = RealGit::new();
    let wt = fixture::spawn_root(ws, id);
    settle(&wt, name, &git).unwrap();
    git.run(&wt, &["commit", "-m", "settle name"]).unwrap();
}

#[test]
fn a_settled_name_reads_back_off_the_ref_and_an_unnamed_agent_reads_as_none() {
    let (_h, ws) = fixture::workspace();
    let git = RealGit::new();
    agent(&ws, ID_A, Some("pale-otter"));
    agent(&ws, ID_B, None);
    assert_eq!(read(&ws, ID_A, &git).as_deref(), Some("pale-otter"));
    assert_eq!(read(&ws, ID_B, &git), None, "an empty name blob is unnamed");
    assert_eq!(read(&ws, "20260103T000000Z-cccccccc", &git), None);
    assert_eq!(
        named(&ws, &git).unwrap(),
        vec![(ID_A.to_string(), "pale-otter".to_string())],
        "the enumeration carries the named agents and only those",
    );
}

#[test]
fn a_fork_never_keeps_the_name_it_inherited() {
    let (_h, ws) = fixture::workspace();
    let git = RealGit::new();
    agent(&ws, ID_A, Some("pale-otter"));
    // A child forks off the named parent's tip, so it inherits the blob;
    // its own dispatch commit settles it (§2.3 step 2).
    let child_id = format!("{ID_A}-20260101T000100Z-dddddddd");
    let child_wt = fixture::spawn_agent(&ws, &child_id, &crate::workspace::agent_ref(ID_A));
    assert_eq!(
        std::fs::read_to_string(child_wt.join(super::NAME_FILE)).unwrap(),
        "pale-otter\n",
        "the fork point's name is what a raw fork carries",
    );
    settle(&child_wt, None, &git).unwrap();
    git.run(&child_wt, &["commit", "-m", "settle name"])
        .unwrap();
    assert_eq!(read(&ws, &child_id, &git), None);
    assert_eq!(
        read(&ws, ID_A, &git).as_deref(),
        Some("pale-otter"),
        "settling the child's name leaves the parent's alone",
    );
}

#[test]
fn a_name_a_living_agent_wears_is_refused_and_the_refusal_names_the_holder() {
    let (_h, ws) = fixture::workspace();
    let git = RealGit::new();
    agent(&ws, ID_A, Some("pale-otter"));
    require_available(&ws, "grey-heron", &git).expect("a free name is available");
    let err = require_available(&ws, "pale-otter", &git).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("\"pale-otter\""), "{msg}");
    assert!(msg.contains(ID_A), "{msg}");
    assert!(matches!(err, Unavailable::Taken { .. }));
}

#[test]
fn a_name_that_is_not_one_unbroken_word_is_refused() {
    let (_h, ws) = fixture::workspace();
    let git = RealGit::new();
    for bad in ["", ".", "..", "a/b", "a\\b", "two words", "trailing\n"] {
        let err = require_available(&ws, bad, &git).unwrap_err();
        assert!(
            matches!(err, Unavailable::Malformed(_)),
            "{bad:?} must be malformed, got {err}",
        );
    }
    let msg = require_available(&ws, "two words", &git)
        .unwrap_err()
        .to_string();
    assert!(msg.contains("single unbroken word"), "{msg}");
}

#[test]
fn a_name_that_begins_like_an_agent_id_is_refused_and_a_near_miss_is_not() {
    let (_h, ws) = fixture::workspace();
    let git = RealGit::new();
    for id_shaped in [ID_A, "20260101T000000Z", "20260101T000000Z-pale-otter"] {
        let err = require_available(&ws, id_shaped, &git).unwrap_err();
        assert!(
            matches!(err, Unavailable::IdShaped(_)),
            "{id_shaped:?} must read as an id, got {err}",
        );
    }
    let msg = require_available(&ws, ID_A, &git).unwrap_err().to_string();
    assert!(msg.contains("YYYYMMDDTHHMMSSZ"), "{msg}");
    // Every way a 16-character head can miss the timestamp shape, plus a
    // length miss — near-misses are ordinary names.
    for near in [
        "2026010XT000000Z-x",  // non-digit in the date
        "20260101X000000Z-x",  // no `T`
        "20260101T00000XZ-x",  // non-digit in the time
        "20260101T000000X-x",  // no `Z`
        "20260101T00000Z-x",   // too short
        "202601011T000000Z-x", // too long
    ] {
        require_available(&ws, near, &git)
            .unwrap_or_else(|e| panic!("{near:?} is an ordinary name, got {e}"));
    }
    assert!(is_id_timestamp("20260101T000000Z"));
}

#[test]
fn a_workspace_the_scan_cannot_read_is_reported_at_creation_and_survived_at_resolution() {
    let holder = tempfile::TempDir::new().unwrap();
    let absent = holder.path().join("not-a-workspace");
    let git = RealGit::new();
    let err = require_available(&absent, "pale-otter", &git).unwrap_err();
    assert!(matches!(err, Unavailable::Scan(_)), "{err}");
    assert!(err.to_string().contains("scan the workspace"), "{err}");
    // Resolution is total: the needle rides through unchanged so the
    // caller's own layout guard speaks.
    assert_eq!(resolve(&absent, "pale-otter", &git).unwrap(), "pale-otter");
}

#[test]
fn resolution_takes_an_id_a_unique_name_or_neither() {
    let (_h, ws) = fixture::workspace();
    let git = RealGit::new();
    agent(&ws, ID_A, Some("pale-otter"));
    agent(&ws, ID_B, None);
    assert_eq!(
        resolve(&ws, ID_A, &git).unwrap(),
        ID_A,
        "an id resolves flat"
    );
    assert_eq!(resolve(&ws, ID_B, &git).unwrap(), ID_B);
    assert_eq!(
        resolve(&ws, "pale-otter", &git).unwrap(),
        ID_A,
        "a unique name resolves to the id that wears it",
    );
    assert_eq!(
        resolve(&ws, "grey-heron", &git).unwrap(),
        "grey-heron",
        "a needle nothing answers to rides through to the existence guard",
    );
}

#[test]
fn a_name_two_living_agents_wear_is_refused_with_both_candidates() {
    let (_h, ws) = fixture::workspace();
    let git = RealGit::new();
    // Creation refuses a duplicate, so the collision is built the way
    // reality can still produce one: a second agent settled with the
    // same name (a fork-back-in off a named commit, §2.3).
    agent(&ws, ID_B, Some("pale-otter"));
    agent(&ws, ID_A, Some("pale-otter"));
    let err = resolve(&ws, "pale-otter", &git).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("ambiguous"), "{msg}");
    assert!(msg.contains(&format!("{ID_A}, {ID_B}")), "{msg}");
}
