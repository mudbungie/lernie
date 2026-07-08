//! Enforcement tests: [`check`] boundaries, [`remaining`] / [`clamp`]
//! clamped inheritance, [`mark_exhausted`], and the `Display` output.

use super::super::{
    Axis, BUDGET_EXHAUSTED_REF_PREFIX, Exhausted, check, clamp, mark_exhausted, remaining,
};
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
fn remaining_depletes_tokens_and_wall_keeps_depth_absolute() {
    let r = repo();
    write_response(r.path(), "conv", 1, &tokens(700));
    write_meta(
        r.path(),
        "conv",
        1,
        "2026-01-01T00:00:00Z",
        "2026-01-01T00:00:20Z",
    );
    let parent = Budgets {
        max_total_tokens: Some(1000),
        max_wall_seconds: Some(60),
        max_depth: Some(4),
    };
    let rem = remaining(r.path(), "conv", &parent);
    assert_eq!(rem.max_total_tokens, Some(300)); // 1000 - 700
    assert_eq!(rem.max_wall_seconds, Some(40)); // 60 - 20
    assert_eq!(rem.max_depth, Some(4)); // absolute, unchanged
}

#[test]
fn remaining_saturates_at_zero_and_passes_none_through() {
    let r = repo();
    write_response(r.path(), "conv", 1, &tokens(5000));
    let parent = Budgets {
        max_total_tokens: Some(1000),
        max_wall_seconds: None,
        max_depth: None,
    };
    let rem = remaining(r.path(), "conv", &parent);
    assert_eq!(rem.max_total_tokens, Some(0)); // saturating_sub, no underflow
    assert_eq!(rem.max_wall_seconds, None); // None passes through
    assert_eq!(rem.max_depth, None);
}

#[test]
fn clamp_takes_min_per_axis_over_all_option_combinations() {
    // A child inherits min(parent_remaining, child_declared) per axis.
    let parent_remaining = Budgets {
        max_total_tokens: Some(300),
        max_wall_seconds: Some(40),
        max_depth: Some(4),
    };
    let child = Budgets {
        max_total_tokens: Some(1000),
        max_wall_seconds: None,
        max_depth: Some(2),
    };
    let c = clamp(&parent_remaining, &child);
    assert_eq!(c.max_total_tokens, Some(300)); // min(300, 1000) → parent's leftover
    assert_eq!(c.max_wall_seconds, Some(40)); // (Some, None) → parent's
    assert_eq!(c.max_depth, Some(2)); // min(4, 2) → child's tighter
    // (None, Some) and (None, None) arms of min_opt:
    let c2 = clamp(&Budgets::default(), &child);
    assert_eq!(c2.max_total_tokens, Some(1000)); // (None, Some) → child's
    assert_eq!(c2.max_wall_seconds, None); // (None, None) → unbounded
}

#[test]
fn mark_exhausted_writes_a_readable_git_native_ref() {
    // Exercise the real `git update-ref` and read the marker back.
    let dir = repo();
    let git = RealGit::new();
    git.run(dir.path(), &["init", "-b", "conv-x"]).unwrap();
    git.run(dir.path(), &["config", "user.email", "b@test.lernie"])
        .unwrap();
    git.run(dir.path(), &["config", "user.name", "b"]).unwrap();
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
