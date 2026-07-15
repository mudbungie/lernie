//! Evaluation statistics (ARCH §9.1).
//!
//! Two metrics, computed from per-task per-run pass/fail outcomes:
//!
//! - **pass@1** (primary, reliability): the mean per-task pass rate —
//!   for each task `#passes / N`, then the mean across tasks
//!   (mean-of-means, N fixed per task). With N fixed this equals the
//!   pooled proportion, so the reported 95% **Wilson score interval** is
//!   computed on the pooled `(successes, trials)` — the natural reading
//!   of "Wilson score intervals on the mean" (§9.1).
//! - **pass@5** (secondary, ceiling capability): the fraction of tasks
//!   for which any of the first five runs passed (§9.1 "any of 5 runs").
//!
//! Per-category breakdowns re-run the same summary over the subset of
//! tasks carrying each of the seven §9.1 tags (a task counts toward every
//! tag it carries). The tag set is derived from the results, never stored
//! separately (`docs/PRINCIPLES.md` Single source of truth).

use std::collections::BTreeSet;

/// z for a two-sided 95% interval.
pub const Z_95: f64 = 1.959_963_984_540_054;

/// pass@5 fixes k = 5 (ARCH §9.1 "any of 5 runs").
pub const PASS_AT_K: usize = 5;

/// One task's per-run pass/fail outcomes plus its category tags.
#[derive(Clone, Debug)]
pub struct TaskResult {
    pub id: String,
    pub categories: Vec<String>,
    pub outcomes: Vec<bool>,
}

impl TaskResult {
    /// Passing runs for this task.
    pub fn passes(&self) -> u64 {
        self.outcomes.iter().filter(|b| **b).count() as u64
    }

    /// pass@5 predicate: any of the first `min(PASS_AT_K, N)` runs passed.
    pub fn any_pass_k(&self) -> bool {
        let k = PASS_AT_K.min(self.outcomes.len());
        self.outcomes[..k].iter().any(|b| *b)
    }
}

/// A closed `[lo, hi]` confidence interval.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interval {
    pub lo: f64,
    pub hi: f64,
}

/// The Wilson score interval for a binomial proportion `successes / n` at
/// the given `z`. A zero-trial input yields the degenerate `[0, 0]`.
pub fn wilson(successes: u64, n: u64, z: f64) -> Interval {
    if n == 0 {
        return Interval { lo: 0.0, hi: 0.0 };
    }
    let n = n as f64;
    let p = successes as f64 / n;
    let z2 = z * z;
    let denom = 1.0 + z2 / n;
    let center = (p + z2 / (2.0 * n)) / denom;
    let margin = (z / denom) * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt();
    Interval {
        lo: center - margin,
        hi: center + margin,
    }
}

/// The pass@1 / pass@5 summary over one set of tasks.
#[derive(Clone, Debug, PartialEq)]
pub struct Summary {
    pub num_tasks: usize,
    pub pass_at_1: f64,
    pub pass_at_1_ci: Interval,
    pub pass_at_5: f64,
}

/// Summarize a set of tasks: mean-of-means pass@1 with a pooled Wilson
/// interval, and the pass@5 task fraction. An empty set is all-zero.
pub fn summarize(tasks: &[&TaskResult]) -> Summary {
    if tasks.is_empty() {
        return Summary {
            num_tasks: 0,
            pass_at_1: 0.0,
            pass_at_1_ci: Interval { lo: 0.0, hi: 0.0 },
            pass_at_5: 0.0,
        };
    }
    let mut rate_sum = 0.0;
    let mut pooled_succ = 0u64;
    let mut pooled_trials = 0u64;
    let mut any_pass = 0usize;
    for t in tasks {
        let n = t.outcomes.len() as u64;
        rate_sum += t.passes() as f64 / n as f64;
        pooled_succ += t.passes();
        pooled_trials += n;
        if t.any_pass_k() {
            any_pass += 1;
        }
    }
    Summary {
        num_tasks: tasks.len(),
        pass_at_1: rate_sum / tasks.len() as f64,
        pass_at_1_ci: wilson(pooled_succ, pooled_trials, Z_95),
        pass_at_5: any_pass as f64 / tasks.len() as f64,
    }
}

/// A category's tag alongside its summary.
#[derive(Clone, Debug, PartialEq)]
pub struct CategoryMetrics {
    pub tag: String,
    pub summary: Summary,
}

/// The full evaluation metrics: the overall summary plus one per category.
#[derive(Clone, Debug, PartialEq)]
pub struct Metrics {
    pub runs_per_task: usize,
    pub overall: Summary,
    pub categories: Vec<CategoryMetrics>,
}

/// Compute the metrics for a completed evaluation. Category tags are the
/// sorted union of tags present across the results.
pub fn compute(results: &[TaskResult]) -> Metrics {
    let runs_per_task = results.first().map(|r| r.outcomes.len()).unwrap_or(0);
    let refs: Vec<&TaskResult> = results.iter().collect();
    let tags: BTreeSet<&str> = results
        .iter()
        .flat_map(|r| r.categories.iter().map(String::as_str))
        .collect();
    let categories = tags
        .into_iter()
        .map(|tag| {
            let subset: Vec<&TaskResult> = results
                .iter()
                .filter(|r| r.categories.iter().any(|c| c == tag))
                .collect();
            CategoryMetrics {
                tag: tag.to_string(),
                summary: summarize(&subset),
            }
        })
        .collect();
    Metrics {
        runs_per_task,
        overall: summarize(&refs),
        categories,
    }
}
