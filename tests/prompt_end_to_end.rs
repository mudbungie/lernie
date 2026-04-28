//! End-to-end subprocess test for `lernie prompt`: chains `lernie
//! new` (scaffold) and `lernie prompt` (one root conversation)
//! against a local `httpmock` server. Asserts the v0.3.1 contract:
//! bare-conv-id branch off `main`, dispatch commit (goal+soul only)
//! before the model call, terminal compaction, `--no-ff` merge back
//! to `main`, the merge=ours discipline (§2.6 — subagent's `summary/`
//! scrubbed from main, reachable only in compactor-branch history),
//! and the diagnostic-only step record at `<conv-repo>/steps/`
//! outside every worktree (§2.3).

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

/// Anthropic-native SSE happy stream; adapter translates to §4.4 JSONL.
const HAPPY_SSE: &str = concat!(
    "event: message_start\n",
    "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_e2e\",\"model\":\"claude-sonnet-4-7\",\"stop_reason\":null,\"content\":[],\"usage\":{\"input_tokens\":2,\"output_tokens\":0}}}\n\n",
    "event: content_block_start\n",
    "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n",
    "event: content_block_delta\n",
    "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"pong\"}}\n\n",
    "event: content_block_stop\n",
    "data: {\"type\":\"content_block_stop\",\"index\":0}\n\n",
    "event: message_delta\n",
    "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":1}}\n\n",
    "event: message_stop\n",
    "data: {\"type\":\"message_stop\"}\n\n",
);

#[test]
fn prompt_subcommand_compacts_and_merges_conversation_to_main() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(200)
            .header("content-type", "text/event-stream")
            .body(HAPPY_SSE);
    });

    let holder = TempDir::new().unwrap();
    let harness = holder.path().join("harness");
    fs::create_dir_all(&harness).unwrap();
    write_global_providers(&harness, &server.base_url());
    let dest = holder.path().join("conv");
    scaffold_repo(&dest, &harness);

    // Main is checked out inside `<conv-repo>/root/` (§2.2).
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
    // Bare conv-id `<ts>-<short-id>`, no `ex/` prefix (§2.3).
    assert!(!branch.contains('/'), "got {branch:?}");
    assert_eq!(branch.len(), 25, "got {branch:?}");
    let conv_id = branch.clone();

    // Main advanced via --no-ff merge (two parent shas).
    let main_head_after = git_capture(&primary, &["rev-parse", "main"]);
    assert_ne!(main_head_before, main_head_after, "main should advance");
    let parents = git_capture(&primary, &["log", "-1", "--pretty=%P", "main"]);
    let parent_shas: Vec<_> = parents.split_whitespace().collect();
    let conv_tip = git_capture(&primary, &["rev-parse", &branch]);
    assert_eq!(parent_shas, [&main_head_before[..], &conv_tip[..]]);

    // §2.6 alignment: summary/** stays on the compactor sub-branch.
    let summary_on_main = git_command(&primary, &["show", "main:summary/001.md"])
        .output()
        .expect("spawn git show");
    assert!(!summary_on_main.status.success(), "summary on main");
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
    assert_eq!(
        summary_blob,
        format!("conversation {conv_id}: terminal compaction")
    );

    // Step records live outside every worktree (§2.2 / §2.3); read
    // them directly from the conv-repo's filesystem.
    let step_dir = dest.join(format!("steps/{conv_id}/001"));
    let read_json = |name: &str| -> serde_json::Value {
        serde_json::from_slice(&fs::read(step_dir.join(name)).unwrap()).unwrap()
    };
    let request = read_json("request.json");
    assert_eq!(request["messages"][0]["content"], "ping");
    assert!(
        request["system"]
            .as_str()
            .unwrap()
            .starts_with("<goal>\nping\n</goal>"),
        "goal not pinned at head of system"
    );
    assert_eq!(request["stream"], true);
    // response.json is JSONL of §4.4 events tail-appended event-by-
    // event; closing the fd is the §3.5 IN_CLOSE_WRITE completion.
    let lines: Vec<serde_json::Value> = fs::read(step_dir.join("response.json"))
        .unwrap()
        .split(|b| *b == b'\n')
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_slice(l).expect("valid JSON line"))
        .collect();
    assert_eq!(lines.first().unwrap()["type"], "message_start");
    assert_eq!(lines.last().unwrap()["type"], "message_stop");
    let text = lines.iter().find(|e| e["type"] == "text_delta").unwrap();
    assert_eq!(text["text"], "pong");
    // meta.json `commit` is the branch tip at step-start (§2.10).
    let commit = read_json("meta.json")["commit"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(commit.len(), 40, "meta.commit not full sha: {commit:?}");
    assert!(commit.chars().all(|c| c.is_ascii_hexdigit()));

    // Step records must not be git-tracked anywhere (§2.2).
    let tracked_steps = git_capture(&primary, &["ls-files", "steps/"]);
    assert!(tracked_steps.is_empty(), "got {tracked_steps:?}");

    // Worktree removed; branch ref survives retention window (§2.3).
    let worktree = dest.join(&conv_id);
    assert!(!worktree.exists(), "conv worktree must be removed");
    let branches = git_capture(&primary, &["branch", "--list", &branch]);
    assert!(
        branches.contains(&branch),
        "conv ref must survive: {branches:?}"
    );

    // §8 unmerged-branch metric: empty post-merge, read from refs.
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
