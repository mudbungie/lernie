//! Rendering the evaluation metrics as a text report (ARCH §9.1, §9.3).
//!
//! The report leads with pass@1 (the optimization target) and its 95%
//! Wilson interval, then pass@5, then the per-category breakdown. The
//! v0.9 baseline criterion — ~40% ± 5% pass@1 on the suite — is printed
//! as a reference line; whether a given run meets it is read off the
//! interval, not asserted here (this crate never runs a live model).

use crate::stats::{Metrics, Summary};

/// Render the full report for one evaluation of `experiment`.
pub fn render(experiment: &str, m: &Metrics) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "experiment: {experiment}\ntasks: {}   runs/task: {}\n\n",
        m.overall.num_tasks, m.runs_per_task
    ));
    out.push_str(&format!(
        "pass@1 (reliability): {}\n",
        pass1_line(&m.overall)
    ));
    out.push_str(&format!(
        "pass@5 (capability):  {:.1}% of tasks\n\n",
        m.overall.pass_at_5 * 100.0
    ));
    out.push_str("per category (§9.1):\n");
    for cat in &m.categories {
        out.push_str(&format!(
            "  {:<20} n={:<3} pass@1 {}  pass@5 {:.1}%\n",
            cat.tag,
            cat.summary.num_tasks,
            pass1_line(&cat.summary),
            cat.summary.pass_at_5 * 100.0
        ));
    }
    out.push_str(
        "\nbaseline criterion (§9.1, v0.9): pass@1 ~40% ± 5% (Wilson CI) on the full suite\n",
    );
    out
}

/// `NN.N% [lo, hi]` (percentages), the shared pass@1 rendering.
fn pass1_line(s: &Summary) -> String {
    format!(
        "{:.1}% [{:.1}%, {:.1}%]",
        s.pass_at_1 * 100.0,
        s.pass_at_1_ci.lo * 100.0,
        s.pass_at_1_ci.hi * 100.0
    )
}
