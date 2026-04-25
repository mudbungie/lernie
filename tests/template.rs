//! Integration test for the conversation repo template, exercised
//! end-to-end through the `lernie new` subcommand.
//!
//! Validates the resulting repo against ARCH §2.2: control-plane files
//! at the conv-repo root (outside any worktree); `root/` as the primary
//! worktree with `.git` inside it and the `merge=ours` `.gitattributes`
//! pinned per ARCH §2.6.

use lernie::config::manifest::{Manifest, OverflowPolicy};
use lernie::config::per_repo_providers::PerRepoProviders;
use lernie::config::version::Version;
use lernie::config::workflow::Workflow;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Git env vars that a hook-invoked test may inherit. They would cause
/// subcommands to operate on the outer repo instead of the scaffolded
/// tempdir, so scrub them from every `git` we spawn here.
const INHERITED_GIT_ENV: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_PREFIX",
    "GIT_COMMON_DIR",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
];

fn scrub_git_env(cmd: &mut Command) {
    for var in INHERITED_GIT_ENV {
        cmd.env_remove(var);
    }
}

fn lernie_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_lernie"))
}

fn scaffold(dest: &Path) -> String {
    let mut cmd = Command::new(lernie_bin());
    scrub_git_env(&mut cmd);
    let out = cmd
        .arg("new")
        .arg(dest)
        .env("GIT_AUTHOR_NAME", "lernie-test")
        .env("GIT_AUTHOR_EMAIL", "test@example.invalid")
        .env("GIT_COMMITTER_NAME", "lernie-test")
        .env("GIT_COMMITTER_EMAIL", "test@example.invalid")
        .output()
        .expect("invoke lernie binary");
    assert!(
        out.status.success(),
        "lernie new failed: {}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

fn scaffolded() -> (TempDir, PathBuf) {
    let holder = TempDir::new().unwrap();
    let dest = holder.path().join("conv");
    let stdout = scaffold(&dest);
    assert_eq!(stdout, dest.display().to_string(), "stdout must echo path");
    (holder, dest)
}

#[test]
fn version_file_is_one() {
    let (_holder, repo) = scaffolded();
    let v = Version::load(&repo.join("version")).unwrap();
    assert_eq!(v, Version(1));
}

#[test]
fn providers_yaml_is_roles_only_and_validates() {
    let (_holder, repo) = scaffolded();
    let (per_repo, warnings) = PerRepoProviders::load(&repo.join("providers.yaml")).unwrap();
    assert!(
        warnings.is_empty(),
        "template providers.yaml carries legacy blocks: {warnings:?}"
    );
    assert!(per_repo.roles.contains_key("worker"));
    assert!(per_repo.roles.contains_key("compactor"));
    assert_eq!(per_repo.roles["worker"].provider, "anthropic");
    assert_eq!(per_repo.roles["worker"].model, "claude-sonnet-4-7");
}

#[test]
fn manifest_yaml_is_role_keyed_per_arch_5_2() {
    let (_holder, repo) = scaffolded();
    let manifest = Manifest::load(&repo.join("manifest.yaml")).unwrap();
    assert!(manifest.roles.contains_key("worker"));
    assert!(manifest.roles.contains_key("compactor"));
    let worker = &manifest.roles["worker"];
    assert!(worker.budget_tokens > 0);
    assert!(worker.pinned.iter().any(|p| p == "goal.md"));
    assert!(worker.pinned.iter().any(|p| p == "soul.md"));
    assert_eq!(worker.overflow, OverflowPolicy::DropOldestSteps);
    assert_eq!(
        manifest.roles["compactor"].overflow,
        OverflowPolicy::Truncate
    );
}

#[test]
fn workflow_yaml_validates() {
    let (_holder, repo) = scaffolded();
    Workflow::load(&repo.join("workflow.yaml")).unwrap();
}

#[test]
fn souls_directory_holds_role_prompts() {
    let (_holder, repo) = scaffolded();
    let souls = repo.join("souls");
    assert!(souls.is_dir());
    assert!(souls.join("worker.md").is_file());
    assert!(souls.join("compactor.md").is_file());
}

#[test]
fn root_worktree_holds_git_and_gitattributes() {
    // ARCH §2.2: `.git` lives in `root/`. ARCH §2.6: `.gitattributes`
    // pins goal.md, soul.md, summary/** to merge=ours, committed on
    // main at scaffold time.
    let (_holder, repo) = scaffolded();
    let root = repo.join("root");
    assert!(root.is_dir());
    assert!(root.join(".git").is_dir());
    let attrs = std::fs::read_to_string(root.join(".gitattributes")).unwrap();
    assert!(attrs.contains("goal.md") && attrs.contains("merge=ours"));
    assert!(attrs.contains("soul.md") && attrs.contains("merge=ours"));
    assert!(attrs.contains("summary/**") && attrs.contains("merge=ours"));
}

#[test]
fn control_plane_lives_outside_any_worktree() {
    // ARCH §2.2 control-plane files (manifest, workflow, providers,
    // version, souls/) sit at the conv-repo root, not inside `root/`.
    // git ls-files inside root/ must not enumerate any of them.
    let (_holder, repo) = scaffolded();
    let root = repo.join("root");
    let mut cmd = Command::new("git");
    scrub_git_env(&mut cmd);
    let out = cmd
        .arg("-C")
        .arg(&root)
        .args(["ls-files"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let listed = String::from_utf8(out.stdout).unwrap();
    for forbidden in [
        "manifest.yaml",
        "workflow.yaml",
        "providers.yaml",
        "version",
        "souls",
    ] {
        assert!(
            !listed.lines().any(|l| l.contains(forbidden)),
            "control file {forbidden} leaked into the worktree: {listed:?}"
        );
    }
    // .gitattributes is the one tracked file.
    assert!(listed.lines().any(|l| l == ".gitattributes"));
}

#[test]
fn root_worktree_runs_main_with_one_commit() {
    let (_holder, repo) = scaffolded();
    let root = repo.join("root");
    let mut log = Command::new("git");
    scrub_git_env(&mut log);
    let log_out = log
        .arg("-C")
        .arg(&root)
        .args(["log", "--oneline"])
        .output()
        .unwrap();
    assert!(log_out.status.success(), "git log failed");
    let text = String::from_utf8(log_out.stdout).unwrap();
    assert_eq!(text.lines().count(), 1, "expected one commit, got:\n{text}");
    assert!(text.contains("init conversation repo"));

    let mut head = Command::new("git");
    scrub_git_env(&mut head);
    let head_out = head
        .arg("-C")
        .arg(&root)
        .args(["symbolic-ref", "--short", "HEAD"])
        .output()
        .unwrap();
    assert!(head_out.status.success());
    let branch = String::from_utf8(head_out.stdout)
        .unwrap()
        .trim()
        .to_string();
    assert_eq!(branch, "main");
}

#[test]
fn no_args_uses_harness_root_with_auto_id() {
    // `lernie new` with no path argument resolves
    // <LERNIE_HOME>/conversations/<auto-id>/ and prints that path.
    let home = TempDir::new().unwrap();
    let mut cmd = Command::new(lernie_bin());
    scrub_git_env(&mut cmd);
    let out = cmd
        .arg("new")
        .env("LERNIE_HOME", home.path())
        .env("GIT_AUTHOR_NAME", "lernie-test")
        .env("GIT_AUTHOR_EMAIL", "test@example.invalid")
        .env("GIT_COMMITTER_NAME", "lernie-test")
        .env("GIT_COMMITTER_EMAIL", "test@example.invalid")
        .output()
        .expect("invoke lernie binary");
    assert!(
        out.status.success(),
        "lernie new (no args) failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let printed = String::from_utf8(out.stdout).unwrap().trim().to_string();
    let printed_path = PathBuf::from(&printed);
    assert!(printed_path.starts_with(home.path().join("conversations")));
    assert!(printed_path.join("manifest.yaml").is_file());
    assert!(printed_path.join("root/.git").is_dir());
}

#[test]
fn binary_refuses_non_empty_destination() {
    let holder = TempDir::new().unwrap();
    let dest = holder.path().join("occupied");
    std::fs::create_dir(&dest).unwrap();
    std::fs::write(dest.join("preexisting"), b"x").unwrap();
    let mut cmd = Command::new(lernie_bin());
    scrub_git_env(&mut cmd);
    let out = cmd.arg("new").arg(&dest).output().unwrap();
    assert!(
        !out.status.success(),
        "binary should refuse non-empty destination"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("not empty"), "unexpected stderr: {stderr}");
}

#[test]
fn binary_accepts_existing_empty_destination() {
    let holder = TempDir::new().unwrap();
    let dest = holder.path().join("preexisting-empty");
    std::fs::create_dir(&dest).unwrap();
    let stdout = scaffold(&dest);
    assert_eq!(stdout, dest.display().to_string());
    assert!(dest.join("manifest.yaml").is_file());
    assert!(dest.join("root/.git").is_dir());
}
