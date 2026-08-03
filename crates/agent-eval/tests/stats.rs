//! Unit coverage for the statistics layer (ARCH §9.1).

use agent_eval::stats::{self, Interval, TaskResult, Z_95};

#[rustfmt::skip]
fn task(id: &str, cats: &[&str], outcomes: &[bool]) -> TaskResult {
    let categories = cats.iter().map(|s| s.to_string()).collect();
    TaskResult { id: id.to_string(), categories, outcomes: outcomes.to_vec() }
}

#[test]
fn passes_and_pass_at_k() {
    let t = task("a", &["x"], &[true, false, true, false, false, true]);
    assert_eq!(t.passes(), 3);
    // any of the first five: runs 0..5 = T,F,T,F,F -> true.
    assert!(t.any_pass_k());
    // fewer than five runs: k clamps to N.
    let short = task("b", &["x"], &[false, false]);
    assert!(!short.any_pass_k());
    let short_pass = task("c", &["x"], &[false, true]);
    assert!(short_pass.any_pass_k());
    // a pass only after run 5 does not count toward pass@5.
    let late = task("d", &["x"], &[false, false, false, false, false, true]);
    assert!(!late.any_pass_k());
}

#[test]
fn wilson_zero_trials_is_degenerate() {
    assert_eq!(stats::wilson(0, 0, Z_95), Interval { lo: 0.0, hi: 0.0 });
}

#[test]
fn wilson_known_value() {
    // 8/10 successes, 95%: Wilson interval ≈ [0.490, 0.943].
    let ci = stats::wilson(8, 10, Z_95);
    assert!((ci.lo - 0.490_2).abs() < 1e-3, "lo={}", ci.lo);
    assert!((ci.hi - 0.943_1).abs() < 1e-3, "hi={}", ci.hi);
    // The point estimate is bracketed.
    assert!(ci.lo < 0.8 && 0.8 < ci.hi);
}

#[test]
fn summarize_empty_is_zero() {
    let s = stats::summarize(&[]);
    assert_eq!(s.num_tasks, 0);
    assert_eq!(s.pass_at_1, 0.0);
    assert_eq!(s.pass_at_5, 0.0);
    assert_eq!(s.pass_at_1_ci, Interval { lo: 0.0, hi: 0.0 });
}

#[test]
fn summarize_mean_of_means() {
    // Two tasks, N=4: rates 1.0 and 0.5 -> mean-of-means 0.75.
    let a = task("a", &["x"], &[true, true, true, true]);
    let b = task("b", &["x"], &[true, true, false, false]);
    let refs = [&a, &b];
    let s = stats::summarize(&refs);
    assert_eq!(s.num_tasks, 2);
    assert!((s.pass_at_1 - 0.75).abs() < 1e-9);
    // pooled 6/8 -> Wilson bracket around 0.75.
    let pooled = stats::wilson(6, 8, Z_95);
    assert_eq!(s.pass_at_1_ci, pooled);
    // both tasks pass at least once -> pass@5 = 1.0.
    assert_eq!(s.pass_at_5, 1.0);
}

#[test]
fn compute_derives_category_union() {
    let a = task("a", &["early_termination"], &[true, false]);
    let b = task(
        "b",
        &["scope_reduction", "early_termination"],
        &[false, false],
    );
    let m = stats::compute(&[a, b]);
    assert_eq!(m.runs_per_task, 2);
    // union of tags, sorted: early_termination, scope_reduction.
    let tags: Vec<&str> = m.categories.iter().map(|c| c.tag.as_str()).collect();
    assert_eq!(tags, ["early_termination", "scope_reduction"]);
    // early_termination covers both tasks; scope_reduction only task b.
    let et = &m.categories[0].summary;
    assert_eq!(et.num_tasks, 2);
    let sr = &m.categories[1].summary;
    assert_eq!(sr.num_tasks, 1);
    // overall pooled: 1 pass of 4 trials.
    assert!((m.overall.pass_at_1 - 0.25).abs() < 1e-9);
}

#[test]
fn compute_empty_results() {
    let m = stats::compute(&[]);
    assert_eq!(m.runs_per_task, 0);
    assert!(m.categories.is_empty());
    assert_eq!(m.overall.num_tasks, 0);
}

#[test]
fn summarize_treats_zero_run_tasks_as_unmeasured_never_nan() {
    // A task with zero runs has no rate (0/0): it joins neither mean.
    let ran = task("ran", &["x"], &[true, false]);
    let unran = task("unran", &["x"], &[]);
    let refs = [&ran, &unran];
    let s = stats::summarize(&refs);
    assert_eq!(s.num_tasks, 2);
    assert!((s.pass_at_1 - 0.5).abs() < 1e-9, "got {}", s.pass_at_1);
    assert_eq!(s.pass_at_5, 1.0);
    // A set holding only unmeasured tasks summarizes all-zero.
    let only = [&unran];
    let s = stats::summarize(&only);
    assert!(s.pass_at_1 == 0.0 && s.pass_at_5 == 0.0);
    assert!(!s.pass_at_1.is_nan());
}
