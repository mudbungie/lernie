//! Coverage for the reproducibility probes (bl-36fa): suite revision,
//! fixture digest, driver version. Fixtures are local temp git repos
//! and fake driver scripts — no network, no model, nothing live.

use agent_eval::repro;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

/// Run a fixture script with the `GIT_*` redirection scrubbed: the
/// test binary may itself run under a git hook (the pre-commit
/// coverage gate), and a leaked `GIT_DIR` would point the fixture's
/// `git init`/`commit` at the outer repository.
fn sh(dir: &Path, script: &str) {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(script).current_dir(dir);
    for var in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_PREFIX",
        "GIT_COMMON_DIR",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    ] {
        cmd.env_remove(var);
    }
    let status = cmd.status().unwrap();
    assert!(status.success(), "fixture script failed: {script}");
}

#[test]
fn suite_revision_reads_head_and_flags_dirt() {
    let dir = tempfile::tempdir().unwrap();
    sh(
        dir.path(),
        "git init -q && printf 'tasks: []\\n' > a.yaml && git add a.yaml && \
         git -c user.email=t@t -c user.name=t commit -qm seed",
    );
    let clean = repro::suite_revision(dir.path()).unwrap();
    assert_eq!(clean.len(), 40); // a bare commit hash, no suffix
    fs::write(dir.path().join("a.yaml"), "tasks: [] # touched\n").unwrap();
    let dirty = repro::suite_revision(dir.path()).unwrap();
    assert_eq!(dirty, format!("{clean}+dirty"));
}

#[test]
fn suite_revision_is_unknown_outside_git() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(repro::suite_revision(dir.path()), None);
}

#[test]
fn fixture_digest_identifies_task_file_content() {
    let a = tempfile::tempdir().unwrap();
    fs::write(a.path().join("b.yaml"), "tasks: []\n").unwrap();
    fs::write(a.path().join("a.yaml"), "tasks: [x]\n").unwrap();
    fs::write(a.path().join("README.md"), "not hashed\n").unwrap();
    let b = tempfile::tempdir().unwrap();
    fs::write(b.path().join("a.yaml"), "tasks: [x]\n").unwrap();
    fs::write(b.path().join("b.yaml"), "tasks: []\n").unwrap();
    // Same task files, different directory and creation order, extra
    // non-yaml file: same identity.
    assert_eq!(
        repro::fixture_digest(a.path()).unwrap(),
        repro::fixture_digest(b.path()).unwrap()
    );
    // Content change changes identity.
    fs::write(b.path().join("a.yaml"), "tasks: [y]\n").unwrap();
    assert_ne!(
        repro::fixture_digest(a.path()).unwrap(),
        repro::fixture_digest(b.path()).unwrap()
    );
    // A missing directory is unknown, never a digest of nothing.
    assert_eq!(repro::fixture_digest(&a.path().join("nope")), None);
    // An unreadable task file (here: a directory in .yaml clothing).
    fs::create_dir(a.path().join("c.yaml")).unwrap();
    assert_eq!(repro::fixture_digest(a.path()), None);
}

fn script(dir: &Path, name: &str, body: &str) -> String {
    let path = dir.join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    path.display().to_string()
}

#[test]
fn driver_version_takes_the_first_reported_line_only() {
    let dir = tempfile::tempdir().unwrap();
    let talks = script(dir.path(), "talks", "echo 'fake-driver 9.9'; echo extra");
    assert_eq!(
        repro::driver_version(&talks),
        Some("fake-driver 9.9".to_string())
    );
}

#[test]
fn driver_version_is_unreported_on_failure_or_silence() {
    let dir = tempfile::tempdir().unwrap();
    let fails = script(dir.path(), "fails", "exit 3");
    assert_eq!(repro::driver_version(&fails), None);
    let silent = script(dir.path(), "silent", "exit 0");
    assert_eq!(repro::driver_version(&silent), None);
    let blank = script(dir.path(), "blank", "echo '   '");
    assert_eq!(repro::driver_version(&blank), None);
    assert_eq!(repro::driver_version("/no/such/driver"), None);
}
