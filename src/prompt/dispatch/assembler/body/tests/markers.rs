//! §5.2 marker-guard tests: the read-path half of the §2.6
//! marker-freedom promise ([`super::super::marked_summary`]) — a
//! `summary/**` entry carrying git's labelled conflict markers refuses
//! composition; everything the promise does not cover composes.

use super::*;

#[test]
fn a_marked_summary_refuses_composition_naming_the_path() {
    // The §2.6 promise, enforced at read time: a summary carrying the
    // labelled markers git writes is a violated invariant, refused
    // loudly — never composed into a model call as if it were context.
    let wt = TempDir::new().unwrap();
    write(
        wt.path(),
        "summary/001.md",
        b"fine\n<<<<<<< HEAD\nours\n=======\ntheirs\n>>>>>>> compactor\n",
    );
    let r = rules(&[], &["summary/**"], 100, OverflowPolicy::Drop);
    let err = compose(wt.path(), Some(&r)).unwrap_err();
    assert!(matches!(
        &err,
        Error::SummaryConflictMarkers { path } if path == "summary/001.md"
    ));
    assert!(err.to_string().contains("summary/001.md"));
}

#[test]
fn a_pinned_marked_summary_is_refused_too() {
    // The guard lives in `select`, shared by `pinned` and `order`: a
    // role that pins the summary chain gets the same refusal — and a
    // lone `>>>>>>> ` remnant (a botched hand-resolution) suffices.
    let wt = TempDir::new().unwrap();
    write(wt.path(), "summary/001.md", b">>>>>>> theirs\n");
    let r = rules(&["summary/**"], &[], 100, OverflowPolicy::Drop);
    assert!(matches!(
        compose(wt.path(), Some(&r)).unwrap_err(),
        Error::SummaryConflictMarkers { .. }
    ));
}

#[test]
fn a_bare_setext_underline_in_a_summary_composes() {
    // git never writes `=======` without the flanking labelled pair,
    // and it is a legitimate markdown setext heading underline —
    // matching it alone would refuse an honest summary.
    let wt = TempDir::new().unwrap();
    write(wt.path(), "summary/001.md", b"Title\n=======\nbody\n");
    let r = rules(&[], &["summary/**"], 100, OverflowPolicy::Drop);
    let out = compose(wt.path(), Some(&r)).unwrap();
    assert_eq!(paths(&out), vec!["summary/001.md"]);
}

#[test]
fn quoted_marker_text_mid_line_in_a_summary_composes() {
    // Only a line *beginning* with a labelled marker is git's; quoted
    // or indented marker text is prose about markers, not markup.
    let wt = TempDir::new().unwrap();
    write(
        wt.path(),
        "summary/001.md",
        b"the diff showed `<<<<<<< HEAD` there\n  >>>>>>> quoted\n",
    );
    let r = rules(&[], &["summary/**"], 100, OverflowPolicy::Drop);
    let out = compose(wt.path(), Some(&r)).unwrap();
    assert_eq!(paths(&out), vec!["summary/001.md"]);
}

#[test]
fn marker_lines_outside_summary_compose_unguarded() {
    // Authored categories carry no §2.6 marker-freedom promise: a
    // skill asset documenting merge conflicts must not hold the branch
    // hostage (the non-UTF-8 argument, `select`).
    let wt = TempDir::new().unwrap();
    write(
        wt.path(),
        "skills/git/SKILL.md",
        b"<<<<<<< HEAD\n=======\n>>>>>>> other\n",
    );
    let r = rules(&[], &["skills/**"], 100, OverflowPolicy::Drop);
    let out = compose(wt.path(), Some(&r)).unwrap();
    assert_eq!(paths(&out), vec!["skills/git/SKILL.md"]);
}
