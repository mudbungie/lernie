//! Coverage for suite loading (ARCH §9.1) — including the real suite.

use agent_eval::suite::{self, SuiteError};
use std::path::PathBuf;

fn write(dir: &std::path::Path, name: &str, body: &str) {
    std::fs::write(dir.join(name), body).unwrap();
}

#[test]
fn loads_and_flattens_sorted() {
    let d = tempfile::tempdir().unwrap();
    // Two yaml files plus a non-yaml file that must be ignored.
    write(
        d.path(),
        "b.yaml",
        "tasks:\n  - id: t2\n    categories: [scope_reduction]\n    prompt: p2\n    check: 'true'\n",
    );
    write(
        d.path(),
        "a.yaml",
        "tasks:\n  - id: t1\n    categories: [early_termination]\n    prompt: p1\n    setup: 'echo hi'\n    check: 'true'\n",
    );
    write(d.path(), "notes.txt", "ignore me");

    let tasks = suite::load(d.path()).unwrap();
    // Sorted by filename: a.yaml (t1) before b.yaml (t2).
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].id, "t1");
    assert_eq!(tasks[0].setup.as_deref(), Some("echo hi"));
    assert_eq!(tasks[1].id, "t2");
    assert_eq!(tasks[1].setup, None);
}

#[test]
fn missing_directory_errors() {
    let err = suite::load(std::path::Path::new("/no/such/suite/dir")).unwrap_err();
    assert!(matches!(err, SuiteError::Dir { .. }));
    assert!(err.to_string().contains("read suite directory"));
}

#[test]
fn parse_error_reports_path() {
    let d = tempfile::tempdir().unwrap();
    write(d.path(), "bad.yaml", "tasks: [oops\n");
    let err = suite::load(d.path()).unwrap_err();
    assert!(matches!(err, SuiteError::Parse { .. }));
    assert!(err.to_string().contains("bad.yaml"));
}

#[test]
fn read_error_on_unreadable_entry() {
    // A directory named `*.yaml` cannot be read as a string, exercising
    // the Read arm without relying on filesystem permissions.
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir(d.path().join("dir.yaml")).unwrap();
    let err = suite::load(d.path()).unwrap_err();
    assert!(matches!(err, SuiteError::Read { .. }));
    assert!(err.to_string().starts_with("read "));
}

#[test]
fn loads_the_real_repo_suite() {
    // The shipped 50-task suite parses through this loader (§9.1).
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/suite");
    let tasks = suite::load(&dir).unwrap();
    assert_eq!(tasks.len(), 50);
    #[rustfmt::skip]
    let ok = tasks.iter().all(|t| !t.prompt.is_empty() && !t.check.is_empty());
    assert!(ok);
}
