//! **The diff rendering, judged over upstream's own frame.** The corpus's
//! `work-diff` fixture carries all four states this surface can be in — a repo
//! it cannot read, a branch that is not there, a cut listing and an attempt
//! that has changed nothing — so the shared assertion is one frame, and what
//! is written by hand here is only what that frame cannot reach.

use serde_json::{Value, json};

use crate::render::rendered;

/// Render a frame and assert every needle is in it.
#[track_caller]
fn says(frame: &Value, needles: &[&str]) {
    let said = rendered(frame);
    for needle in needles {
        assert!(said.contains(needle), "{frame}\n-> {said}");
    }
}

/// **A work diff is a diff and not a row of counters** (bl-2e1d). The three
/// facts a diff rendering has that this one did not: the two ends the
/// comparison actually resolved to, a totals line, and an elision that says it
/// is one.
#[test]
fn a_work_diff_says_its_two_ends_its_totals_and_where_it_was_cut() {
    let path = crate::test_support::corpus::root()
        .join("answers")
        .join("work-diff.json");
    let frame = crate::test_support::corpus::fixture(&path)
        .frames
        .first()
        .cloned()
        .expect("the work-diff fixture carries a frame");
    says(
        &frame,
        &[
            // The header's own content: what was asked for, and what it read.
            "→ main  from work/bl-3  at aaa..bbb",
            // Folded over the churn, with the binary counted apart because no
            // line count describes it.
            "2 files  +3 -1  1 binary",
            "src/a.rs  +3 -1",
            "assets/x.png  binary",
            // The cut, which was decoded and dropped before this ball.
            "… the engine stopped listing here",
            // A `diff` row with no files is the attempt having written
            // nothing, and not a row with no listing.
            "nothing changed on this branch yet",
            // The other two absences keep their own words.
            "missing work/bl-2",
        ],
    );
    // A row that answered no listing carries no body: nothing is indented
    // under it, so an unreadable repo is not a change of size zero.
    let said = rendered(&frame);
    assert!(!said.contains("unreadable\n    "), "{said}");
}

/// **A half-named comparison is not one**, and a file that carries no line
/// count contributes none — the two arms the corpus's frame does not reach.
#[test]
fn one_end_alone_names_no_comparison_and_a_binary_only_row_totals_nothing() {
    let said = rendered(&json!({"ok": true, "kind": "work-diff", "rows": [{
        "ball_id": "bl-9", "project": "p", "state": "diff",
        "target": "main", "source": "work/bl-9", "target_oid": "aaa",
        "truncated": false,
        "files": [{"path": "logo.png", "binary": true}]}]}));
    assert!(!said.contains(" at "), "{said}");
    assert!(said.contains("1 file  1 binary"), "{said}");
    assert!(said.contains("logo.png  binary"), "{said}");
    assert!(!said.contains('+'), "{said}");
}

/// An empty listing is the fact, and the empty arm is this listing's own.
#[test]
fn a_wall_whose_agents_changed_nothing_says_so() {
    says(
        &json!({"ok": true, "kind": "work-diff", "rows": []}),
        &["nothing has changed"],
    );
}
