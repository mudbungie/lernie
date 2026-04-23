//! End-to-end subprocess test for `lernie prompt` (v0.2).
//!
//! Chains the two binaries — `lernie new` to scaffold a conversation
//! repo, then `lernie prompt` to drive one exchange — against a local
//! `httpmock` server standing in for the Anthropic endpoint.
//!
//! The v0.2 contract is ARCH §2.3 + §2.6 + §2.7: an exchange is its
//! own branch off `main`, with the snapshot commit before the model
//! call, a response follow-up commit, terminal compaction, and a
//! `--no-ff` merge back to `main`. The test asserts:
//!
//! - stdout is the exchange branch name (`ex/<ts>-<short-id>`).
//! - main's HEAD advanced to a merge commit.
//! - The merge commit's second parent is the compacted exchange tip.
//! - `.agent/compactions/001.md` is reachable from main's HEAD and
//!   carries the terminal response text.
//! - `exchanges/<id>/steps/001/{request,response}.json` are reachable
//!   from main's HEAD (the compactor is a stub — deletion of step
//!   dirs is v0.3 work per ARCH §12).
//! - The exchange worktree under `.lernie/worktrees/ex/…` is removed.

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

fn adapter_bin_dir() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_lernie-provider-anthropic"))
        .parent()
        .expect("bin path has a parent")
}

fn path_env_with_adapter() -> std::ffi::OsString {
    let existing = std::env::var_os("PATH").unwrap_or_default();
    let mut dirs = vec![adapter_bin_dir().to_path_buf()];
    dirs.extend(std::env::split_paths(&existing));
    std::env::join_paths(dirs).expect("PATH join")
}

/// Scaffold a conversation repo at `dest` and rewrite its
/// `providers.yaml` so the `anthropic` provider's `endpoint:` points
/// at the local mock. The rewrite is committed on top of the
/// scaffold's initial commit so the next `lernie prompt` invocation
/// has a clean tree to extend.
fn scaffold_with_endpoint(dest: &Path, endpoint: &str) {
    let out = Command::new(lernie_bin())
        .arg("new")
        .arg(dest)
        .output()
        .expect("spawn lernie new");
    assert!(
        out.status.success(),
        "lernie new: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let providers_path = dest.join(".agent/providers.yaml");
    let original = fs::read_to_string(&providers_path).unwrap();
    fs::write(
        &providers_path,
        original.replace("https://api.anthropic.com", endpoint),
    )
    .unwrap();
    let commit = git_command(
        dest,
        &["commit", "-am", "test: point providers.yaml at mock"],
    )
    .output()
    .expect("spawn git commit");
    assert!(
        commit.status.success(),
        "git commit: {}",
        String::from_utf8_lossy(&commit.stderr)
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
fn prompt_subcommand_compacts_and_merges_exchange_to_main() {
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
    let dest = holder.path().join("conv");
    scaffold_with_endpoint(&dest, &server.base_url());

    let main_head_before = git_capture(&dest, &["rev-parse", "main"]);
    assert!(!main_head_before.is_empty());

    let prompt_out = Command::new(lernie_bin())
        .arg("prompt")
        .arg(&dest)
        .arg("ping")
        .env("PATH", path_env_with_adapter())
        .env("ANTHROPIC_API_KEY", "test-key")
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
    assert!(
        branch.starts_with("ex/"),
        "expected ex/<ts>-<id> branch name, got {branch:?}"
    );
    let exchange_id = branch.strip_prefix("ex/").unwrap().to_string();

    // Main advanced: HEAD is a merge commit. `git log -1 --pretty=%P
    // main` returns two parent shas — that's the --no-ff shape.
    let main_head_after = git_capture(&dest, &["rev-parse", "main"]);
    assert_ne!(main_head_before, main_head_after, "main should advance");
    let parents = git_capture(&dest, &["log", "-1", "--pretty=%P", "main"]);
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
    // Second parent is the compacted exchange tip.
    let ex_tip = git_capture(&dest, &["rev-parse", &branch]);
    assert_eq!(
        parent_shas[1], ex_tip,
        "second parent should be the exchange branch's tip"
    );

    // Compaction summary reachable from main's HEAD and carries the
    // terminal response text.
    let summary = git_capture(&dest, &["show", "main:.agent/compactions/001.md"]);
    assert_eq!(summary, format!("exchange {exchange_id}: pong"));

    // Step artifacts reachable from main (v0.2 stub does not prune).
    let request_blob = git_capture(
        &dest,
        &[
            "show",
            &format!("main:exchanges/{exchange_id}/steps/001/request.json"),
        ],
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
        &dest,
        &[
            "show",
            &format!("main:exchanges/{exchange_id}/steps/001/response.json"),
        ],
    );
    let response: serde_json::Value = serde_json::from_str(&response_blob).unwrap();
    assert_eq!(response["assistant_response"], "pong");
    assert_eq!(response["provider"], "anthropic");

    // Exchange worktree has been removed after the merge; the branch
    // ref survives for the retention window (§2.3).
    let worktree = dest.join(".lernie/worktrees/ex").join(&exchange_id);
    assert!(
        !worktree.exists(),
        "exchange worktree should be removed after merge"
    );
    let branches = git_capture(&dest, &["branch", "--list", &branch]);
    assert!(
        branches.contains(&branch),
        "exchange branch ref should survive merge: {branches:?}"
    );

    // No unmerged exchange branches: the merge-back is the one that
    // moves a branch from "unmerged" to "merged", so post-prompt
    // `git branch --list ex/* --no-merged main` is empty. This is the
    // unmerged-branch-count metric (§8) read directly from git refs —
    // no sidecar JSON required (PRINCIPLES.md "Single source of
    // truth").
    let unmerged = git_capture(&dest, &["branch", "--list", "ex/*", "--no-merged", "main"]);
    assert!(
        unmerged.is_empty(),
        "no ex/* branches should remain unmerged; got {unmerged:?}"
    );
}

#[test]
fn prompt_subcommand_surfaces_missing_repo() {
    let holder = TempDir::new().unwrap();
    let out = Command::new(lernie_bin())
        .arg("prompt")
        .arg(holder.path().join("does-not-exist"))
        .arg("hi")
        .env("PATH", path_env_with_adapter())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn lernie prompt");
    assert!(!out.status.success(), "expected failure on missing repo");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("lernie prompt"), "got: {stderr}");
}
