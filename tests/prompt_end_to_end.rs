//! End-to-end subprocess test for `lernie prompt`.
//!
//! Chains the two v0.1 binaries — `lernie new` to scaffold a conversation
//! repo, then `lernie prompt` to drive one exchange — against a local
//! `httpmock` server standing in for the Anthropic endpoint. Exercises the
//! seams the in-process unit tests cannot reach: argv parsing, PATH lookup
//! of `lernie-provider-anthropic`, subprocess stdin/stdout piping, env-var
//! propagation, the on-disk layout produced by `lernie new`, and the real
//! `git` binary landing the commit.
//!
//! The flow follows ARCH §12's success criterion: "A single prompt is sent
//! to a provider endpoint, the response is written to disk, and is visible
//! in the conversation repo as a commit."

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
/// hook context. When they are set, they override `-C` and redirect the
/// child `git` back to the outer repo — which breaks tests that create
/// throwaway repos in temp dirs. Scrub them on every `git` spawn here.
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
    // `env!("CARGO_BIN_EXE_lernie-provider-anthropic")` is an absolute path
    // into the target directory; we want the containing dir so we can
    // prepend it to PATH for the child `lernie` process's adapter lookup.
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

/// Scaffold a conversation repo at `dest` and rewrite its `providers.yaml`
/// so the `anthropic` provider's `endpoint:` points at the local mock. The
/// rewrite is committed on top of the scaffold's initial commit so the
/// next `lernie prompt` invocation has a clean tree to extend.
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
fn prompt_subcommand_writes_exchange_and_commits() {
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

    // Run `lernie prompt` with PATH carrying the built adapter.
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

    let sha = String::from_utf8(prompt_out.stdout)
        .unwrap()
        .trim()
        .to_string();
    assert!(!sha.is_empty(), "expected non-empty SHA on stdout");

    // Exchange record on disk. Filter out the template's `.gitkeep`.
    let entries: Vec<_> = fs::read_dir(dest.join("exchanges"))
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .collect();
    assert_eq!(entries.len(), 1, "expected exactly one exchange file");
    let body: serde_json::Value =
        serde_json::from_slice(&fs::read(entries[0].path()).unwrap()).unwrap();
    assert_eq!(body["assistant_response"], "pong");
    assert_eq!(body["user_message"], "ping");
    assert_eq!(body["provider"], "anthropic");

    // Commit exists with the printed SHA, and its tree contains the exchange.
    let rev = git_command(&dest, &["rev-parse", "HEAD"])
        .output()
        .expect("git rev-parse");
    assert_eq!(
        String::from_utf8_lossy(&rev.stdout).trim(),
        sha,
        "HEAD SHA mismatch"
    );
    let show = git_command(&dest, &["show", "--name-only", "--pretty=format:", &sha])
        .output()
        .expect("git show");
    let files = String::from_utf8_lossy(&show.stdout);
    assert!(
        files.contains("exchanges/"),
        "exchange file not in commit: {files}"
    );
}

#[test]
fn prompt_subcommand_surfaces_missing_repo() {
    // No scaffold → providers.yaml absent → prompt exits non-zero.
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
