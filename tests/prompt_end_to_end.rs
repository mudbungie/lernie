//! End-to-end subprocess test for `lernie prompt` (v0.3): chains
//! `lernie new` (scaffold) and `lernie prompt` (one root conversation)
//! against a local `httpmock` server. Asserts the v0.3 contract from
//! ARCH §2.3 + §2.6 + §2.7 — bare-conv-id branch off `main`, snapshot
//! commit before the model call, response follow-up, terminal
//! compaction, and `--no-ff` merge back to main. Also asserts the
//! merge=ours discipline (§2.6) holds: subagent's `summary/` is
//! scrubbed from main but reachable in compactor-branch history;
//! `steps/<conv-id>/` crosses up into main as designed.

use httpmock::Method::POST;
use httpmock::MockServer;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use tempfile::TempDir;

fn lernie_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lernie")
}

/// Git env vars a pre-commit hook context may inherit; they would
/// override `-C` and redirect to the outer repo. Scrubbed on spawn.
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

/// Build the sibling adapter binary (separate workspace crate, so
/// `CARGO_BIN_EXE_<name>` is not set) and return its directory.
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

/// Write a global `<harness-root>/providers.yaml` (ARCH §2.2/§4.1)
/// pointing the `anthropic` endpoint at `endpoint`.
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

/// Scaffold a conversation repo at `dest` via `lernie new`.
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
    // Bare conv-id `<ts>-<short-id>`: 16-char ts + `-` + 8 hex
    // chars (ARCH §2.3). No `ex/` prefix.
    assert!(!branch.contains('/'), "got {branch:?}");
    assert_eq!(branch.len(), 25, "got {branch:?}");
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

    // ARCH §2.6: summary/** is pinned to the parent's pre-merge
    // state via the alignment step in `rebase_and_merge`. Main and
    // the conv branch's tip are both clean of `summary/`; the
    // compactor sub-branch's history retains it for provenance.
    let summary_on_main = Command::new("git")
        .args([
            "-C",
            primary.to_str().unwrap(),
            "show",
            "main:summary/001.md",
        ])
        .output()
        .expect("spawn git show");
    assert!(
        !summary_on_main.status.success(),
        "summary/001.md must not be on main"
    );
    let summary_commits = git_capture(
        &primary,
        &[
            "log",
            "--all",
            "--pretty=%H",
            "--diff-filter=A",
            "--",
            "summary/001.md",
        ],
    );
    let first_sha = summary_commits.lines().next().expect("summary in history");
    let summary_blob = git_capture(&primary, &["show", &format!("{first_sha}:summary/001.md")]);
    assert_eq!(summary_blob, format!("conversation {conv_id}: pong"));

    // Step artifacts reachable from main (`steps/<sub-id>/` is not
    // merge=ours per ARCH §2.6 — those records do cross up).
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
    assert_eq!(response["content"][0]["text"], "pong");
    assert_eq!(response["provider"], "anthropic");

    // Worktree removed after merge; branch ref survives the
    // retention window (§2.3).
    let worktree = dest.join(&conv_id);
    assert!(!worktree.exists(), "conv worktree must be removed");
    let branches = git_capture(&primary, &["branch", "--list", &branch]);
    assert!(
        branches.contains(&branch),
        "conv ref must survive: {branches:?}"
    );

    // Unmerged-branch metric (§8): post-prompt the conv branch has
    // moved from unmerged → merged, so `git branch --no-merged main`
    // is empty. Read directly from refs, no sidecar (PRINCIPLES.md).
    let unmerged = git_capture(
        &primary,
        &["branch", "--list", "*-*", "--no-merged", "main"],
    );
    assert!(unmerged.is_empty(), "got {unmerged:?}");
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
