//! Integration test for the conversation repo template, exercised
//! end-to-end through the `lernie new` subcommand.
//!
//! Validates the resulting repo against every config schema declared in
//! `docs/ARCHITECTURE.md` §2.2. Any pinned path from `manifest.yaml`
//! that is written at dispatch time (§2.8) and therefore legitimately
//! absent from a fresh scaffold is listed in [`WRITTEN_AT_DISPATCH`].

use lernie::config::agents::Agents;
use lernie::config::cross::{check_agents_against_providers, check_workflow_against_agents};
use lernie::config::manifest::Manifest;
use lernie::config::providers::Providers;
use lernie::config::version::Version;
use lernie::config::workflow::Workflow;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Pinned manifest paths that ARCH §2.8 declares as written at dispatch
/// time. They are allowed to be absent from a freshly scaffolded repo.
const WRITTEN_AT_DISPATCH: &[&str] = &[".agent/goal.md"];

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

fn scaffold(dest: &Path) {
    let mut cmd = Command::new(lernie_bin());
    scrub_git_env(&mut cmd);
    let out = cmd
        .arg("new")
        .arg(dest)
        // Pin git identity so the initial commit does not depend on the
        // runner's global config.
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
}

fn scaffolded() -> (TempDir, PathBuf) {
    let holder = TempDir::new().unwrap();
    let dest = holder.path().join("conv");
    scaffold(&dest);
    (holder, dest)
}

#[test]
fn version_file_is_one() {
    let (_holder, repo) = scaffolded();
    let v = Version::load(&repo.join(".agent/version")).unwrap();
    assert_eq!(v, Version(1));
}

#[test]
fn providers_yaml_validates() {
    let (_holder, repo) = scaffolded();
    let (providers, warnings) = Providers::load(&repo.join(".agent/providers.yaml")).unwrap();
    assert!(
        warnings.is_empty(),
        "template providers.yaml produced warnings: {warnings:?}"
    );
    assert!(providers.providers.contains_key("anthropic"));
    assert!(providers.models.contains_key("claude-sonnet-4-7"));
    assert!(providers.models.contains_key("claude-haiku-4-5"));
}

#[test]
fn agents_yaml_validates_and_resolves_against_providers() {
    let (_holder, repo) = scaffolded();
    let agents = Agents::load(&repo.join(".agent/agents.yaml")).unwrap();
    let (providers, _) = Providers::load(&repo.join(".agent/providers.yaml")).unwrap();
    check_agents_against_providers(&agents, &providers).unwrap();
    assert!(agents.agents.contains_key("worker"));
    assert!(agents.agents.contains_key("compactor"));
}

#[test]
fn manifest_yaml_validates() {
    let (_holder, repo) = scaffolded();
    let manifest = Manifest::load(&repo.join(".agent/manifest.yaml")).unwrap();
    assert!(manifest.context.budget_tokens > 0);
    assert!(!manifest.context.pinned.is_empty());
}

#[test]
fn workflow_yaml_validates_and_dispatch_roles_resolve() {
    let (_holder, repo) = scaffolded();
    let workflow = Workflow::load(&repo.join(".agent/workflow.yaml")).unwrap();
    let agents = Agents::load(&repo.join(".agent/agents.yaml")).unwrap();
    check_workflow_against_agents(&workflow, &agents).unwrap();
}

#[test]
fn prompts_referenced_by_agents_exist_on_disk() {
    let (_holder, repo) = scaffolded();
    let agents = Agents::load(&repo.join(".agent/agents.yaml")).unwrap();
    let system = repo.join(".agent/system");
    for (role, definition) in &agents.agents {
        let resolved = system.join(&definition.system_prompt);
        assert!(
            resolved.is_file(),
            "agent {role}: system_prompt {} missing at {}",
            definition.system_prompt.display(),
            resolved.display(),
        );
    }
}

#[test]
fn manifest_pinned_paths_resolve_or_are_written_at_dispatch() {
    let (_holder, repo) = scaffolded();
    let manifest = Manifest::load(&repo.join(".agent/manifest.yaml")).unwrap();
    for pinned in &manifest.context.pinned {
        if WRITTEN_AT_DISPATCH.contains(&pinned.as_str()) {
            continue;
        }
        let resolved = repo.join(pinned);
        assert!(
            resolved.exists(),
            "pinned path {pinned} must exist or be listed as written-at-dispatch"
        );
    }
}

#[test]
fn goal_md_is_absent_from_freshly_scaffolded_repo() {
    let (_holder, repo) = scaffolded();
    let goal = repo.join(".agent/goal.md");
    assert!(
        !goal.exists(),
        ".agent/goal.md must not be in the template; it is written at dispatch (ARCH §2.8)"
    );
}

#[test]
fn state_files_are_present_and_empty_baseline() {
    let (_holder, repo) = scaffolded();
    let events = repo.join(".agent/state/events.log");
    assert!(events.is_file());
    let events_body = std::fs::read_to_string(&events).unwrap();
    assert!(events_body.is_empty());
}

#[test]
fn skill_and_tool_dirs_exist_but_are_empty() {
    let (_holder, repo) = scaffolded();
    for sub in [".agent/system/skills", ".agent/system/tools"] {
        let dir = repo.join(sub);
        assert!(dir.is_dir(), "{sub} should be a directory");
        let has_non_gitkeep = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .any(|e| e.file_name() != ".gitkeep");
        assert!(
            !has_non_gitkeep,
            "{sub} should be empty except for .gitkeep"
        );
    }
}

#[test]
fn scaffolded_repo_has_one_commit() {
    let (_holder, repo) = scaffolded();
    let mut cmd = Command::new("git");
    scrub_git_env(&mut cmd);
    let out = cmd
        .arg("-C")
        .arg(&repo)
        .args(["log", "--oneline"])
        .output()
        .unwrap();
    assert!(out.status.success(), "git log failed");
    let text = String::from_utf8(out.stdout).unwrap();
    assert_eq!(
        text.lines().count(),
        1,
        "expected exactly one commit, got:\n{text}"
    );
    assert!(text.contains("init conversation repo"));
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
    // create_dir before running — task description says refuse only if
    // the dest is non-empty, so a pre-created empty dir must work.
    let holder = TempDir::new().unwrap();
    let dest = holder.path().join("preexisting-empty");
    std::fs::create_dir(&dest).unwrap();
    scaffold(&dest);
    assert!(dest.join(".agent/version").is_file());
}
