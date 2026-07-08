//! Integration tests: `lernie stop` idempotence + error paths
//! (companion to `tests/stop_cli.rs`'s cascade test).

mod stop_common;

use std::fs;
use std::process::{Command, Stdio};
use stop_common::{git_command, git_run, lernie_bin, scaffold_repo, write_global_models};
use tempfile::TempDir;

#[test]
fn stop_on_branch_with_no_live_writer_is_idempotent_success() {
    let holder = TempDir::new().unwrap();
    let harness = holder.path().join("harness");
    fs::create_dir_all(&harness).unwrap();
    write_global_models(&harness);
    let dest = holder.path().join("conv");
    scaffold_repo(&dest, &harness);
    let primary = dest.join("root");
    // Diverge from main: branch off main + add a commit so the new
    // tip is not reachable from main. Just creating the branch
    // would leave it == main and `--is-ancestor` would call it
    // merged.
    git_run(&primary, &["checkout", "-b", "stale-branch-22"]);
    fs::write(primary.join("scratch.txt"), "diverge\n").unwrap();
    git_run(&primary, &["add", "scratch.txt"]);
    git_run(
        &primary,
        &[
            "-c",
            "user.email=t@e",
            "-c",
            "user.name=T",
            "commit",
            "-m",
            "diverge",
        ],
    );
    git_run(&primary, &["checkout", "main"]);

    let stop_out = Command::new(lernie_bin())
        .arg("stop")
        .arg(&dest)
        .arg("stale-branch-22")
        .stderr(Stdio::piped())
        .output()
        .expect("spawn lernie stop");
    assert!(
        stop_out.status.success(),
        "lernie stop must succeed idempotently: {}",
        String::from_utf8_lossy(&stop_out.stderr)
    );
}

#[test]
fn stop_on_missing_branch_errors() {
    let holder = TempDir::new().unwrap();
    let harness = holder.path().join("harness");
    fs::create_dir_all(&harness).unwrap();
    write_global_models(&harness);
    let dest = holder.path().join("conv");
    scaffold_repo(&dest, &harness);

    let out = Command::new(lernie_bin())
        .arg("stop")
        .arg(&dest)
        .arg("does-not-exist")
        .stderr(Stdio::piped())
        .output()
        .expect("spawn lernie stop");
    assert!(!out.status.success(), "expected nonzero exit");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("does-not-exist") && stderr.contains("does not exist"),
        "got: {stderr}"
    );
}

#[test]
fn stop_on_merged_branch_errors() {
    let holder = TempDir::new().unwrap();
    let harness = holder.path().join("harness");
    fs::create_dir_all(&harness).unwrap();
    write_global_models(&harness);
    let dest = holder.path().join("conv");
    scaffold_repo(&dest, &harness);
    let primary = dest.join("root");

    // Merged branch == any ancestor of main; main itself qualifies.
    let out = Command::new(lernie_bin())
        .arg("stop")
        .arg(&dest)
        .arg("main")
        .stderr(Stdio::piped())
        .output()
        .expect("spawn lernie stop");
    assert!(!out.status.success(), "expected nonzero exit");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("\"main\"") && stderr.contains("already merged"),
        "got: {stderr}"
    );
    let s = git_command(&primary, &["merge-base", "--is-ancestor", "main", "main"])
        .status()
        .expect("spawn git");
    assert!(s.success());
}
