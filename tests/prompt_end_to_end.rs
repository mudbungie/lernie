//! End-to-end subprocess test for `lernie prompt` (v0.3).
//!
//! Chains the two binaries — `lernie new` to scaffold a conversation
//! repo, then `lernie prompt` to drive one root conversation —
//! against a local `httpmock` server standing in for the Anthropic
//! endpoint.
//!
//! The v0.3 contract is ARCH §2.3 + §2.6 + §2.7: a root conversation
//! is its own bare-`<conv-id>` branch off `main` (no `ex/` prefix),
//! with the snapshot commit before the model call, a response
//! follow-up commit, terminal compaction, and a `--no-ff` merge back
//! to `main`. The merge runs in the primary worktree at
//! `<conv-repo>/root/`. The test asserts:
//!
//! - stdout is the conversation branch name (`<ts>-<short-id>`).
//! - main's HEAD advanced to a merge commit.
//! - The merge commit's second parent is the compacted conversation
//!   tip.
//! - `summary/001.md` is reachable from main's HEAD and carries the
//!   terminal response text.
//! - `steps/<conv-id>/001/{request,response}.json` are reachable from
//!   main's HEAD (the compactor is a stub — deletion of step dirs is
//!   v0.4 work per ARCH §12).
//! - The conversation worktree at `<conv-repo>/<conv-id>/` is removed
//!   after the merge.

use httpmock::Method::POST;
use httpmock::MockServer;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use tempfile::TempDir;

fn lernie_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lernie")
}

/// Environment variables a `git` subprocess inherits from a pre-commit
/// hook context. When they are set, they override `-C` and redirect
/// the child `git` back to the outer repo — which breaks tests that
/// create throwaway repos in temp dirs. Scrub them on every `git`
/// spawn here.
const INHERITED_GIT_ENV: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_PREFIX",
    "GIT_COMMON_DIR",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
];

fn git_command(dest: &Path, args: &[&str]) -> Command {
    let mut cmd = Command::new("git");
    for var in INHERITED_GIT_ENV {
        cmd.env_remove(var);
    }
    cmd.arg("-C").arg(dest).args(args);
    cmd
}

/// Build the sibling adapter binary (if not already built) and return the
/// directory it lives in. The binary is in a separate workspace crate, so
/// `CARGO_BIN_EXE_<name>` is not set for this test — we build it via cargo
/// at test start and locate it via the workspace's `target/<profile>/`.
fn adapter_bin_dir() -> std::path::PathBuf {
    static BUILT: std::sync::Once = std::sync::Once::new();
    BUILT.call_once(|| {
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
        let status = std::process::Command::new(cargo)
            .args([
                "build",
                "--quiet",
                "--package",
                "lernie-provider-anthropic",
                "--bin",
                "lernie-provider-anthropic",
            ])
            .status()
            .expect("spawn cargo build");
        assert!(
            status.success(),
            "cargo build lernie-provider-anthropic failed"
        );
    });
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(profile)
}

fn path_env_with_adapter() -> std::ffi::OsString {
    let existing = std::env::var_os("PATH").unwrap_or_default();
    let mut dirs = vec![adapter_bin_dir()];
    dirs.extend(std::env::split_paths(&existing));
    std::env::join_paths(dirs).expect("PATH join")
}

/// Lay out a temp harness root (ARCH §2.2) with a global
/// `providers.yaml` whose `anthropic` endpoint points at `endpoint`.
/// The per-repo `providers.yaml` (created by `lernie new`) only
/// carries role assignments; endpoint and auth live globally per
/// ARCH §4.1.
fn write_global_providers(harness: &Path, endpoint: &str) {
    let yaml = format!(
        "providers:\n  \
           anthropic:\n    \
             endpoint: {endpoint}\n    \
             auth:\n      type: api_key\n      env: ANTHROPIC_API_KEY\n\
         models:\n  \
           claude-sonnet-4-7:\n    \
             provider: anthropic\n    \
             model_id: claude-sonnet-4-7\n    \
             capabilities: [tool_use_native]\n    \
             context_window: 200000\n  \
           claude-haiku-4-5:\n    \
             provider: anthropic\n    \
             model_id: claude-haiku-4-5\n    \
             capabilities: [tool_use_native]\n    \
             context_window: 200000\n",
    );
    fs::write(harness.join("providers.yaml"), yaml).unwrap();
}

/// Scaffold a conversation repo at `dest` via `lernie new`. The
/// per-repo `providers.yaml` from the embedded template carries the
/// role → (provider, model) mapping; endpoint and auth come from the
/// global `<harness-root>/providers.yaml`.
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

fn git_capture(dest: &Path, args: &[&str]) -> String {
    let out = git_command(dest, args).output().expect("spawn git");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

#[test]
fn prompt_subcommand_compacts_and_merges_conversation_to_main() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(200).body(
            r#"{"id":"msg_e2e","model":"claude-sonnet-4-7","stop_reason":"end_turn",
               "content":[{"type":"text","text":"pong"}],
               "usage":{"input_tokens":2,"output_tokens":1}}"#,
        );
    });

    let holder = TempDir::new().unwrap();
    let harness = holder.path().join("harness");
    fs::create_dir_all(&harness).unwrap();
    write_global_providers(&harness, &server.base_url());
    let dest = holder.path().join("conv");
    scaffold_repo(&dest, &harness);

    // Main is checked out inside `<conv-repo>/root/` per the v0.3
    // layout (ARCH §2.2). The pre-prompt main HEAD is the scaffold's
    // initial commit on that branch.
    let primary = dest.join("root");
    let main_head_before = git_capture(&primary, &["rev-parse", "main"]);
    assert!(!main_head_before.is_empty());

    let prompt_out = Command::new(lernie_bin())
        .arg("prompt")
        .arg(&dest)
        .arg("ping")
        .env("PATH", path_env_with_adapter())
        .env("ANTHROPIC_API_KEY", "test-key")
        .env("LERNIE_HOME", &harness)
        .stderr(Stdio::piped())
        .output()
        .expect("spawn lernie prompt");
    assert!(
        prompt_out.status.success(),
        "lernie prompt: {}",
        String::from_utf8_lossy(&prompt_out.stderr)
    );

    let branch = String::from_utf8(prompt_out.stdout)
        .unwrap()
        .trim()
        .to_string();
    // Branch is the bare conv-id `<ts>-<short-id>`: 16-char compact
    // timestamp + `-` + 8 hex chars (ARCH §2.3). No `ex/` prefix.
    assert!(
        !branch.contains('/'),
        "expected bare conv-id, got {branch:?}"
    );
    assert_eq!(
        branch.len(),
        25,
        "expected 16-char ts + '-' + 8-char hex, got {branch:?}"
    );
    let conv_id = branch.clone();

    // Main advanced: HEAD is a merge commit. `git log -1 --pretty=%P
    // main` returns two parent shas — that's the --no-ff shape.
    let main_head_after = git_capture(&primary, &["rev-parse", "main"]);
    assert_ne!(main_head_before, main_head_after, "main should advance");
    let parents = git_capture(&primary, &["log", "-1", "--pretty=%P", "main"]);
    let parent_shas: Vec<_> = parents.split_whitespace().collect();
    assert_eq!(
        parent_shas.len(),
        2,
        "main HEAD should be a merge commit with two parents; got {parents:?}"
    );
    assert_eq!(
        parent_shas[0], main_head_before,
        "first parent should be the pre-prompt main"
    );
    // Second parent is the compacted conversation tip.
    let conv_tip = git_capture(&primary, &["rev-parse", &branch]);
    assert_eq!(
        parent_shas[1], conv_tip,
        "second parent should be the conversation branch's tip"
    );

    // Compaction summary reachable from main's HEAD and carries the
    // terminal response text.
    let summary = git_capture(&primary, &["show", "main:summary/001.md"]);
    assert_eq!(summary, format!("conversation {conv_id}: pong"));

    // Step artifacts reachable from main (v0.3 stub does not prune).
    let request_blob = git_capture(
        &primary,
        &["show", &format!("main:steps/{conv_id}/001/request.json")],
    );
    let request: serde_json::Value = serde_json::from_str(&request_blob).unwrap();
    assert_eq!(request["messages"][0]["content"], "ping");
    assert!(
        request["system"]
            .as_str()
            .unwrap()
            .starts_with("<goal>\nping\n</goal>"),
        "goal not pinned at head of system"
    );
    let response_blob = git_capture(
        &primary,
        &["show", &format!("main:steps/{conv_id}/001/response.json")],
    );
    let response: serde_json::Value = serde_json::from_str(&response_blob).unwrap();
    assert_eq!(response["assistant_response"], "pong");
    assert_eq!(response["provider"], "anthropic");

    // Conversation worktree has been removed after the merge; the
    // branch ref survives for the retention window (§2.3).
    let worktree = dest.join(&conv_id);
    assert!(
        !worktree.exists(),
        "conversation worktree should be removed after merge"
    );
    let branches = git_capture(&primary, &["branch", "--list", &branch]);
    assert!(
        branches.contains(&branch),
        "conversation branch ref should survive merge: {branches:?}"
    );

    // No unmerged conversation branches: the merge-back is the one
    // that moves a branch from "unmerged" to "merged", so post-prompt
    // `git branch --list '*-*' --no-merged main` is empty. This is
    // the unmerged-branch-count metric (§8) read directly from git
    // refs — no sidecar JSON required (PRINCIPLES.md "Single source
    // of truth").
    let unmerged = git_capture(
        &primary,
        &["branch", "--list", "*-*", "--no-merged", "main"],
    );
    assert!(
        unmerged.is_empty(),
        "no conversation branches should remain unmerged; got {unmerged:?}"
    );
}

#[test]
fn prompt_subcommand_surfaces_missing_repo() {
    let holder = TempDir::new().unwrap();
    let harness = holder.path().join("harness");
    fs::create_dir_all(&harness).unwrap();
    write_global_providers(&harness, "https://api.anthropic.com");
    let out = Command::new(lernie_bin())
        .arg("prompt")
        .arg(holder.path().join("does-not-exist"))
        .arg("hi")
        .env("PATH", path_env_with_adapter())
        .env("LERNIE_HOME", &harness)
        .stderr(Stdio::piped())
        .output()
        .expect("spawn lernie prompt");
    assert!(!out.status.success(), "expected failure on missing repo");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("lernie prompt"), "got: {stderr}");
}
