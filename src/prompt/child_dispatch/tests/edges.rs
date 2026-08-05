//! Spawn-edge failures at the dispatch fork ([`super::super::run`]).
//! Split from `tests.rs` for the 300-line repo cap; the launcher stub,
//! the request builder and the seeded mint RNG are shared from there.

use super::*;

#[test]
fn a_duplicate_sub_id_surfaces_as_worktree_add() {
    // Two dispatches with the same fixed sub-id collide at `worktree
    // add` — the structural id-uniqueness guarantee (§2.3).
    struct FixedClock;
    impl Clock for FixedClock {
        fn now_iso8601(&self) -> String {
            "2026-01-01T00:00:00Z".into()
        }
        fn now_compact(&self) -> String {
            "ct9".into()
        }
    }
    struct FixedIdGen;
    impl IdGen for FixedIdGen {
        fn short(&self) -> String {
            "feedface".into()
        }
    }
    let (_h, ws) = fixture::workspace();
    let parent_wt = fixture::spawn_root(&ws, "20260101-p1");
    let g = crate::template::RealGit::new();
    let launcher = RecordingLauncher::ok();
    run(
        &req(&ws, "20260101-p1", &parent_wt, "g"),
        &g,
        &FixedClock,
        &FixedIdGen,
        &launcher,
        &test_rng(),
    )
    .unwrap();
    let err = run(
        &req(&ws, "20260101-p1", &parent_wt, "g"),
        &g,
        &FixedClock,
        &FixedIdGen,
        &launcher,
        &test_rng(),
    )
    .unwrap_err();
    assert!(
        matches!(
            err,
            Error::Git {
                op: "worktree add",
                ..
            }
        ),
        "got {err:?}"
    );
}
