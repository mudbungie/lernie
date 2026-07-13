//! Executor-lock wiring in the shipped executor (ARCH §2.11): the step
//! loop acquires the branch's inbox lease at start, and a driver that
//! loses the acquire exits as a clean no-op (Writer/driver totality).

use super::fixtures::*;
use super::stubs::AdapterReply;
use crate::prompt::Error;
use crate::prompt::inbox::{inbox_dir, try_acquire};

/// The deterministic conv-id under the standard fixtures (FixedClock
/// `ct-1`, FixedIdGen `deadbeef`).
const CONV_ID: &str = "ct-1-deadbeef";

/// Version-guard-only adapter: no model stream is scripted, so any model
/// call would panic. A no-op driver must not reach one.
fn version_only_adapter() -> StubAdapter {
    StubAdapter::scripted([AdapterReply::Ok(version_line())])
}

#[test]
fn driver_that_loses_the_acquire_is_a_clean_noop() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    // Simulate a live executor already driving this branch by holding
    // its inbox lease across the run.
    let _held = try_acquire(&inbox_dir(repo.path(), CONV_ID))
        .unwrap()
        .expect("fresh inbox is acquirable");

    let adapter = version_only_adapter();
    let git = StubGit::ok();
    let branch = run_with_stubs(repo.path(), "hello", &adapter, &git).unwrap();

    // The verb still reports the branch name, but nothing was driven:
    // no worktree spawned, no model call issued.
    assert_eq!(branch, CONV_ID);
    // Control resolution (read-only, §2.2) ran; no branch work did.
    assert!(
        git.runs.borrow().iter().all(|(_, a)| a[0] != "worktree"),
        "no git branch work on the no-op path"
    );
    assert!(
        !worktree_path(repo.path()).exists(),
        "no worktree materialized"
    );
}

#[test]
fn acquire_failure_surfaces_executorlock_error() {
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    // Block the inbox home so create_dir_all of `inbox/<conv-id>` fails.
    std::fs::write(repo.path().join("inbox"), b"not a dir").unwrap();

    let adapter = version_only_adapter();
    let git = StubGit::ok();
    let err = run_with_stubs(repo.path(), "hello", &adapter, &git).unwrap_err();
    assert!(matches!(err, Error::ExecutorLock { .. }), "{err}");
}

#[test]
fn fresh_root_prompt_wins_its_lease_and_drives() {
    // The common path: a unique conv-id means the lease is free, so the
    // executor acquires and proceeds to spawn the branch and step.
    // (Release-on-exit is covered deterministically by the lock unit
    // test `lock_releases_on_drop`.)
    let repo = scaffold_repo(VALID_PER_REPO_PROVIDERS_YAML, Some("body"));
    let adapter = StubAdapter::happy(&happy_response_bytes());
    let git = StubGit::ok();
    let branch = run_with_stubs(repo.path(), "hello", &adapter, &git).unwrap();
    assert_eq!(branch, CONV_ID);
    assert!(!git.runs.borrow().is_empty(), "the branch was driven");
}
