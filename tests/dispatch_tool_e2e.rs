//! End-to-end test for the v0.4 Phase 2 `dispatch` built-in.
//!
//! Drives the production `lernie tool dispatch` shim with the env vars
//! the executor sets (ARCH §3.3): `LERNIE_CONV_REPO` pointing at a
//! freshly-scaffolded conv-repo, `LERNIE_CONV_BRANCH` pointing at a
//! fabricated parent branch. The dispatch tool is expected to spawn
//! `lernie dispatch worker` (Phase 1's CLI), capture the new sub-branch
//! name, and emit `{"status":"in_progress","handle":"<sub-branch>"}`
//! on its own stdout. Asserts the §2.5 "Async work uses handles"
//! contract on a real subprocess tree, not a stub.

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use tempfile::TempDir;

fn lernie_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lernie")
}

const INHERITED_GIT_ENV: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_PREFIX",
    "GIT_COMMON_DIR",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
];

fn git_run(dest: &Path, args: &[&str]) {
    let mut cmd = Command::new("git");
    for var in INHERITED_GIT_ENV {
        cmd.env_remove(var);
    }
    let out = cmd
        .arg("-C")
        .arg(dest)
        .args(args)
        .output()
        .expect("spawn git");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn scaffold_repo(dest: &Path, harness: &Path) {
    let out = Command::new(lernie_bin())
        .arg("new")
        .arg(dest)
        .env("LERNIE_HOME", harness)
        .output()
        .expect("spawn lernie new");
    assert!(
        out.status.success(),
        "lernie new: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn fabricate_parent(repo: &Path, parent_branch: &str) {
    let primary = repo.join("root");
    let parent_wt = repo.join(parent_branch);
    git_run(
        &primary,
        &[
            "worktree",
            "add",
            "-b",
            parent_branch,
            parent_wt.to_str().unwrap(),
            "main",
        ],
    );
}

#[test]
fn dispatch_tool_returns_handle_and_spawns_worker_branch() {
    let holder = TempDir::new().unwrap();
    let harness = holder.path().join("harness");
    fs::create_dir_all(&harness).unwrap();
    let repo = holder.path().join("conv");
    scaffold_repo(&repo, &harness);
    let parent_branch = "p1-conv";
    fabricate_parent(&repo, parent_branch);

    // Run `lernie tool dispatch` with the env shape the executor sets.
    let input = serde_json::json!({
        "role": "worker",
        "goal": "summarize the parent branch's commits"
    })
    .to_string();
    let mut child = Command::new(lernie_bin())
        .args(["tool", "dispatch"])
        .env("LERNIE_HOME", &harness)
        .env("LERNIE_CONV_REPO", &repo)
        .env("LERNIE_CONV_BRANCH", parent_branch)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn lernie tool dispatch");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().expect("wait lernie tool dispatch");
    assert!(
        out.status.success(),
        "exit={:?} stderr={}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );

    let payload: serde_json::Value = serde_json::from_slice(&out.stdout).expect("stdout JSON");
    assert_eq!(payload["status"], "in_progress");
    let handle = payload["handle"].as_str().expect("handle is string");
    assert!(
        handle.starts_with(&format!("{parent_branch}-")),
        "handle should descend from parent: {handle}"
    );

    // The subagent worktree was actually created at the handle path
    // (Phase 1's CLI does the worktree allocation + dispatch commit).
    assert!(
        repo.join(handle).exists(),
        "subagent worktree must exist at {}",
        repo.join(handle).display()
    );

    // The handle is also a branch ref — Phase 1 lands the dispatch
    // commit, so the ref points one commit past `parent_branch`.
    let primary = repo.join("root");
    let parent_tip = run_git_capture(&primary, &["rev-parse", parent_branch]);
    let sub_tip = run_git_capture(&primary, &["rev-parse", handle]);
    assert_ne!(parent_tip, sub_tip, "subagent tip must advance");
}

#[test]
fn dispatch_tool_surfaces_unknown_role_as_nonzero_with_stderr_message() {
    let holder = TempDir::new().unwrap();
    let harness = holder.path().join("harness");
    fs::create_dir_all(&harness).unwrap();
    let repo = holder.path().join("conv");
    scaffold_repo(&repo, &harness);
    fabricate_parent(&repo, "p1");

    let input = serde_json::json!({ "role": "verifier", "goal": "g" }).to_string();
    let mut child = Command::new(lernie_bin())
        .args(["tool", "dispatch"])
        .env("LERNIE_HOME", &harness)
        .env("LERNIE_CONV_REPO", &repo)
        .env("LERNIE_CONV_BRANCH", "p1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn lernie tool dispatch");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().expect("wait lernie tool dispatch");
    assert!(!out.status.success(), "expected non-zero exit");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("verifier") && stderr.contains("not defined"),
        "stderr: {stderr}"
    );
}

fn run_git_capture(dest: &Path, args: &[&str]) -> String {
    let mut cmd = Command::new("git");
    for var in INHERITED_GIT_ENV {
        cmd.env_remove(var);
    }
    let out = cmd
        .arg("-C")
        .arg(dest)
        .args(args)
        .output()
        .expect("spawn git");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}
