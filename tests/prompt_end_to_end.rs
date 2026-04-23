//! End-to-end subprocess test for `lernie prompt` (v0.2).
//!
//! Chains the two binaries — `lernie new` to scaffold a conversation
//! repo, then `lernie prompt` to drive one exchange — against a local
//! `httpmock` server standing in for the Anthropic endpoint.
//!
//! The v0.2 contract is ARCH §2.3: an exchange is its own branch off
//! `main`, with the snapshot commit before the model call and the
//! response on a follow-up commit. The test asserts:
//!
//! - stdout is the branch name (`ex/<ts>-<short-id>`).
//! - main's HEAD is unchanged by the prompt invocation.
//! - The exchange branch exists and is two commits ahead of main.
//! - `.agent/goal.md` is on the branch with the user message as body.
//! - `exchanges/<id>/steps/001/{request,response}.json` are present on
//!   the branch with the expected shape.

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

#[test]
fn prompt_subcommand_spawns_exchange_branch_off_main() {
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

    let main_head_before = String::from_utf8(
        git_command(&dest, &["rev-parse", "main"])
            .output()
            .expect("pre-invoke git rev-parse")
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string();
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

    // Main's HEAD is unchanged.
    let main_head_after = String::from_utf8(
        git_command(&dest, &["rev-parse", "main"])
            .output()
            .expect("post-invoke git rev-parse")
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string();
    assert_eq!(
        main_head_before, main_head_after,
        "main advanced — v0.2 must leave it alone"
    );

    // Exchange branch is exactly two commits ahead of main.
    let rev_list = git_command(&dest, &["rev-list", "--count", &format!("main..{branch}")])
        .output()
        .expect("git rev-list");
    let count = String::from_utf8(rev_list.stdout)
        .unwrap()
        .trim()
        .to_string();
    assert_eq!(count, "2", "expected snapshot + response commits");

    let exchange_id = branch.strip_prefix("ex/").unwrap();
    let worktree = dest.join(".lernie/worktrees/ex").join(exchange_id);

    let goal = fs::read_to_string(worktree.join(".agent/goal.md")).unwrap();
    assert_eq!(goal, "ping");

    let step_dir = worktree.join(format!("exchanges/{exchange_id}/steps/001"));
    let request: serde_json::Value =
        serde_json::from_slice(&fs::read(step_dir.join("request.json")).unwrap()).unwrap();
    assert_eq!(request["messages"][0]["content"], "ping");
    assert!(
        request["system"]
            .as_str()
            .unwrap()
            .starts_with("<goal>\nping\n</goal>"),
        "goal not pinned at head of system"
    );
    let response: serde_json::Value =
        serde_json::from_slice(&fs::read(step_dir.join("response.json")).unwrap()).unwrap();
    assert_eq!(response["assistant_response"], "pong");
    assert_eq!(response["provider"], "anthropic");

    // Unmerged branches are enumerable via git refs directly.
    let ex_branches = git_command(&dest, &["branch", "--list", "ex/*"])
        .output()
        .expect("git branch --list");
    let listed = String::from_utf8(ex_branches.stdout).unwrap();
    // `git branch` prefixes checked-out branches with `*` and
    // worktree-checked-out branches with `+`. Strip both so the
    // comparison is against bare branch names.
    let names: Vec<&str> = listed
        .lines()
        .map(|line| line.trim_start_matches(['*', '+', ' ']).trim())
        .filter(|s| !s.is_empty())
        .collect();
    assert_eq!(
        names,
        vec![branch.as_str()],
        "exactly one open exchange branch"
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
