//! The settle-the-name pre-flight at the dispatch fork (§2.3, yog
//! bl-aca4): a supplied name is worn verbatim, an omitted one is minted
//! — deterministically under the injected RNG — and no fork ends
//! nameless. Split from `tests.rs` for the 300-line repo cap.

use super::*;
use crate::workspace::agent_name;
use crate::workspace::agent_name::mint::{SplitMix64, mint};

#[test]
fn an_omitted_name_is_minted_at_the_fork_deterministically() {
    let (_h, ws) = fixture::workspace();
    let parent_wt = fixture::spawn_root(&ws, "20260101-p1");
    let g = crate::template::RealGit::new();
    // The fresh workspace has no named agents, so the pre-flight's
    // occupied set is empty: the fork's mint and this prediction are the
    // same pure function of the same seed (§2.3).
    let predicted = mint(&SplitMix64::from_seed(11), &Default::default()).unwrap();
    let child = run(
        &req(&ws, "20260101-p1", &parent_wt, "g"),
        &g,
        &crate::prompt::SystemClock,
        &crate::prompt::NanoIdGen,
        &RecordingLauncher::ok(),
        &SplitMix64::from_seed(11),
    )
    .unwrap();
    assert_eq!(
        agent_name::read(&ws, &child, &g).as_deref(),
        Some(predicted.as_str()),
        "the omitted name settles as the mint's word — no fork ends nameless",
    );
    // The minted word now occupies the pool: a second omission-dispatch
    // under the same seed scans past it to the next word.
    let sibling = run(
        &req(&ws, "20260101-p1", &parent_wt, "g"),
        &g,
        &crate::prompt::SystemClock,
        &crate::prompt::NanoIdGen,
        &RecordingLauncher::ok(),
        &SplitMix64::from_seed(11),
    )
    .unwrap();
    let worn = agent_name::read(&ws, &sibling, &g).expect("the sibling is named too");
    assert_ne!(worn, predicted, "a living name is never minted twice");
}

#[test]
fn a_supplied_name_is_worn_verbatim_and_a_taken_one_forks_nothing() {
    let (_h, ws) = fixture::workspace();
    let parent_wt = fixture::spawn_root(&ws, "20260101-p1");
    let g = crate::template::RealGit::new();
    let named = ChildDispatchRequest {
        name: Some("pale-otter"),
        ..req(&ws, "20260101-p1", &parent_wt, "g")
    };
    let child = run(
        &named,
        &g,
        &crate::prompt::SystemClock,
        &crate::prompt::NanoIdGen,
        &RecordingLauncher::ok(),
        &test_rng(),
    )
    .unwrap();
    assert_eq!(
        agent_name::read(&ws, &child, &g).as_deref(),
        Some("pale-otter"),
        "a supplied name is the settled name — the mint never overrides it",
    );
    // The same name again is the unchanged uniqueness refusal (§2.3):
    // pre-flighted, so nothing forked.
    let before = workspace::agent_ids(&ws, &g).unwrap().len();
    let err = run(
        &named,
        &g,
        &crate::prompt::SystemClock,
        &crate::prompt::NanoIdGen,
        &RecordingLauncher::ok(),
        &test_rng(),
    )
    .unwrap_err();
    assert!(matches!(err, Error::NameUnavailable(_)), "got {err:?}");
    assert_eq!(
        workspace::agent_ids(&ws, &g).unwrap().len(),
        before,
        "a refused name leaves no branch behind",
    );
}
