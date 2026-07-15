//! Coverage for the report renderer (ARCH §9.1, §9.3).

use agent_eval::report;
use agent_eval::stats::{self, TaskResult};

#[rustfmt::skip]
fn task(id: &str, cats: &[&str], outcomes: &[bool]) -> TaskResult {
    let categories = cats.iter().map(|s| s.to_string()).collect();
    TaskResult { id: id.to_string(), categories, outcomes: outcomes.to_vec() }
}

#[test]
fn render_includes_all_sections() {
    let results = vec![
        task(
            "a",
            &["early_termination"],
            &[true, true, false, false, true],
        ),
        task(
            "b",
            &["scope_reduction"],
            &[false, false, false, false, false],
        ),
    ];
    let m = stats::compute(&results);
    let text = report::render("baseline", &m);

    assert!(text.contains("experiment: baseline"));
    assert!(text.contains("tasks: 2"));
    assert!(text.contains("runs/task: 5"));
    assert!(text.contains("pass@1 (reliability):"));
    assert!(text.contains("pass@5 (capability):"));
    assert!(text.contains("per category (§9.1):"));
    assert!(text.contains("early_termination"));
    assert!(text.contains("scope_reduction"));
    assert!(text.contains("baseline criterion (§9.1, v0.9)"));
    // pass@1 line carries a percentage and a bracketed interval.
    assert!(text.contains('%'));
    assert!(text.contains('['));
}
