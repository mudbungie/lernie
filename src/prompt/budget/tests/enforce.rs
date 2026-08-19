//! Enforcement tests: [`check`] boundaries (incl. the whole-tree
//! derivation over the root id for a subagent branch), [`mark_exhausted`],
//! and the `Display` output.

use super::super::{Axis, BUDGET_EXHAUSTED_REF_PREFIX, Exhausted, check, mark_exhausted};
use super::{repo, seg, usage_line, write_meta, write_response};
use crate::config::Budgets;
use crate::template::{GitRunner, RealGit};

fn tokens(n: u32) -> String {
    seg(&usage_line(Some(n), None, None, None))
}

#[test]
fn check_unbounded_never_triggers() {
    let r = repo();
    write_response(r.path(), "conv", 1, &tokens(1_000_000));
    // All limits None → no trigger at huge spend or any depth.
    assert_eq!(check(r.path(), "a-b-c-d-e", &Budgets::default()), None);
}

#[test]
fn check_tokens_boundary_below_at_and_over() {
    let r = repo();
    write_response(r.path(), "conv", 1, &tokens(8));
    let below = Budgets {
        max_total_tokens: Some(9),
        ..Budgets::default()
    };
    assert_eq!(check(r.path(), "conv", &below), None); // 8 < 9
    let at = Budgets {
        max_total_tokens: Some(8),
        ..Budgets::default()
    };
    assert_eq!(
        check(r.path(), "conv", &at),
        Some(Exhausted {
            axis: Axis::Tokens,
            limit: 8,
            actual: 8
        })
    ); // 8 >= 8 → exhausted (stop before overspending)
    let over = Budgets {
        max_total_tokens: Some(5),
        ..Budgets::default()
    };
    assert_eq!(check(r.path(), "conv", &over).unwrap().axis, Axis::Tokens);
}

#[test]
fn check_falls_through_axes_in_order() {
    let r = repo();
    write_response(r.path(), "conv", 1, &tokens(4));
    write_meta(
        r.path(),
        "conv",
        1,
        "2026-01-01T00:00:00Z",
        "2026-01-01T00:00:50Z",
    );
    // tokens has headroom (4 < 100); wall trips (50 >= 40); depth unset.
    let b = Budgets {
        max_total_tokens: Some(100),
        max_wall_seconds: Some(40),
        max_depth: None,
    };
    let ex = check(r.path(), "conv", &b).unwrap();
    assert_eq!(ex.axis, Axis::Wall);
    assert_eq!((ex.limit, ex.actual), (40, 50));
    // Below the wall limit → no trigger.
    let ok = Budgets {
        max_wall_seconds: Some(51),
        ..Budgets::default()
    };
    assert_eq!(check(r.path(), "conv", &ok), None);
}

#[test]
fn check_depth_is_positional_allows_at_limit_exhausts_over() {
    let r = repo();
    // "p-q-r-s-t" → 4 hyphens / 2 = depth 2.
    let branch = "p-q-r-s-t";
    let at = Budgets {
        max_depth: Some(2),
        ..Budgets::default()
    };
    assert_eq!(check(r.path(), branch, &at), None); // depth 2, max 2 → allowed
    let over = Budgets {
        max_depth: Some(1),
        ..Budgets::default()
    };
    let ex = check(r.path(), branch, &over).unwrap();
    assert_eq!(ex.axis, Axis::Depth);
    assert_eq!((ex.limit, ex.actual), (1, 2)); // 2 > 1 → exhausted
}

#[test]
fn check_derives_whole_tree_over_root_for_a_subagent_branch() {
    // `steps/` is one shared tree (ARCH §2.2/§2.3/§2.6): a subagent driver
    // must see the root's and its siblings' live spend, not just its own
    // subtree — this is the invariant the no-inheritance refactor rests on.
    let r = repo();
    write_response(r.path(), "t0-r0", 1, &tokens(600)); // the root's spend
    write_response(r.path(), "t0-r0-t1-c1", 1, &tokens(500)); // this subagent's
    write_response(r.path(), "t0-r0-t1-s2", 1, &tokens(400)); // a sibling's
    let b = Budgets {
        max_total_tokens: Some(1000),
        ..Budgets::default()
    };
    // Checked from the subagent branch: root_of → "t0-r0", summing the
    // whole tree = 600 + 500 + 400 = 1500 >= 1000. The subagent's own
    // subtree alone (500) is under 1000, so this can only trip whole-tree.
    let ex = check(r.path(), "t0-r0-t1-c1", &b).unwrap();
    assert_eq!(ex.axis, Axis::Tokens);
    assert_eq!((ex.limit, ex.actual), (1000, 1500));
}

#[test]
fn check_wall_also_derives_over_the_whole_tree() {
    // Wall likewise sums the whole tree, not just the driver's subtree.
    let r = repo();
    write_meta(
        r.path(),
        "t0-r0",
        1,
        "2026-01-01T00:00:00Z",
        "2026-01-01T00:00:30Z",
    );
    write_meta(
        r.path(),
        "t0-r0-t1-c1",
        1,
        "2026-01-01T00:00:00Z",
        "2026-01-01T00:00:25Z",
    );
    let b = Budgets {
        max_wall_seconds: Some(50),
        ..Budgets::default()
    };
    // From the subagent: 30 + 25 = 55 >= 50; the subagent's own 25 is under.
    let ex = check(r.path(), "t0-r0-t1-c1", &b).unwrap();
    assert_eq!(ex.axis, Axis::Wall);
    assert_eq!((ex.limit, ex.actual), (50, 55));
}

#[test]
fn check_depth_exhausts_a_deep_subagent_but_not_its_root() {
    // Depth is positional and per-driver: the same limit spares the root
    // (depth 0) and exhausts a subagent below max_depth.
    let r = repo();
    let b = Budgets {
        max_depth: Some(1),
        ..Budgets::default()
    };
    assert_eq!(check(r.path(), "t0-r0", &b), None); // depth 0 <= 1
    let ex = check(r.path(), "t0-r0-t1-c1-t2-g1", &b).unwrap(); // depth 2
    assert_eq!(ex.axis, Axis::Depth);
    assert_eq!((ex.limit, ex.actual), (1, 2));
}

#[test]
fn mark_exhausted_writes_a_readable_git_native_ref() {
    // Exercise the real `git update-ref` and read the marker back.
    let dir = repo();
    let git = RealGit::new();
    git.run(dir.path(), &["init", "-b", "conv-x"]).unwrap();
    git.run(dir.path(), &["config", "user.email", "b@test.invalid"])
        .unwrap();
    git.run(dir.path(), &["config", "user.name", "b"]).unwrap();
    git.run(dir.path(), &["config", "core.hooksPath", "/dev/null"])
        .unwrap();
    git.run(dir.path(), &["commit", "--allow-empty", "-m", "base"])
        .unwrap();
    mark_exhausted(dir.path(), "conv-x", &git).unwrap();
    let ref_name = format!("{BUDGET_EXHAUSTED_REF_PREFIX}conv-x");
    let out = git
        .run_capture(dir.path(), &["for-each-ref", &ref_name])
        .unwrap();
    assert!(out.contains(&ref_name), "ref not written; got {out:?}");
}

#[test]
fn mark_exhausted_surfaces_git_failure() {
    // A plain tempdir is not a git repo, so `git update-ref` errors.
    let dir = repo();
    let git = RealGit::new();
    assert!(mark_exhausted(dir.path(), "conv-x", &git).is_err());
}

#[test]
fn display_names_axis_and_ratio() {
    let e = Exhausted {
        axis: Axis::Tokens,
        limit: 8,
        actual: 8,
    };
    assert_eq!(e.to_string(), "max_total_tokens exhausted (8/8)");
    assert_eq!(Axis::Wall.to_string(), "max_wall_seconds");
    assert_eq!(Axis::Depth.to_string(), "max_depth");
}
