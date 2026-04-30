//! Shared helpers for the `lernie stop` integration tests
//! (`tests/stop_*.rs`). Lives at `tests/stop_common/mod.rs` so cargo
//! treats it as a module rather than a separate test binary; each
//! consumer pulls it in with `mod stop_common;`.
//!
//! Each integration test crate compiles its own copy of this module
//! and only references a subset of the helpers; `dead_code` is
//! silenced at the module level rather than per-fn.

#![allow(dead_code)]

use std::fs;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub fn lernie_bin() -> &'static str {
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

pub fn git_command(dest: &Path, args: &[&str]) -> Command {
    let mut cmd = Command::new("git");
    for var in INHERITED_GIT_ENV {
        cmd.env_remove(var);
    }
    cmd.arg("-C").arg(dest).args(args);
    cmd
}

pub fn git_run(dest: &Path, args: &[&str]) {
    let out = git_command(dest, args).output().expect("spawn git");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn adapter_bin_dir() -> std::path::PathBuf {
    static BUILT: std::sync::Once = std::sync::Once::new();
    BUILT.call_once(|| {
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
        let status = Command::new(cargo)
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
        assert!(status.success(), "cargo build adapter failed");
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

pub fn path_env_with_adapter() -> std::ffi::OsString {
    let existing = std::env::var_os("PATH").unwrap_or_default();
    let mut dirs = vec![adapter_bin_dir()];
    dirs.extend(std::env::split_paths(&existing));
    std::env::join_paths(dirs).expect("PATH join")
}

pub fn write_global_providers(harness: &Path, endpoint: &str) {
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

pub fn scaffold_repo(dest: &Path, harness: &Path) {
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

/// Block until the conversation branch exists in `<repo>/root/`'s
/// refs, or the deadline expires. The branch name is the bare conv-id
/// (`<ts>-<short-id>`, 25 chars per §2.3) — the only ref of that
/// shape in a fresh repo. Panics on early prompt exit (so the caller
/// sees the prompt's stderr) or on timeout.
pub fn poll_for_conv_branch_with_diag(
    primary: &Path,
    deadline: Duration,
    prompt_child: &mut Child,
) -> String {
    let until = Instant::now() + deadline;
    loop {
        if let Some(name) = scan_conv_branches(primary) {
            return name;
        }
        if let Ok(Some(status)) = prompt_child.try_wait() {
            let mut buf = Vec::new();
            if let Some(mut e) = prompt_child.stderr.take() {
                use std::io::Read as _;
                let _ = e.read_to_end(&mut buf);
            }
            panic!(
                "lernie prompt exited early: {status:?}; stderr: {}",
                String::from_utf8_lossy(&buf)
            );
        }
        if Instant::now() >= until {
            let buf = drain_stderr(prompt_child);
            let branches = git_command(primary, &["branch", "--list"])
                .output()
                .expect("spawn git");
            let _ = prompt_child.kill();
            let _ = prompt_child.wait();
            panic!(
                "timeout waiting for conversation branch under {}; branches: {:?}; stderr: {}",
                primary.display(),
                String::from_utf8_lossy(&branches.stdout),
                String::from_utf8_lossy(&buf)
            );
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn drain_stderr(child: &mut Child) -> Vec<u8> {
    use std::io::Read as _;
    use std::os::fd::AsRawFd;
    let Some(stderr) = child.stderr.as_mut() else {
        return Vec::new();
    };
    let fd = stderr.as_raw_fd();
    let prev_flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    unsafe { libc::fcntl(fd, libc::F_SETFL, prev_flags | libc::O_NONBLOCK) };
    let mut buf = Vec::new();
    let _ = stderr.read_to_end(&mut buf);
    unsafe { libc::fcntl(fd, libc::F_SETFL, prev_flags) };
    buf
}

fn scan_conv_branches(primary: &Path) -> Option<String> {
    let out = git_command(primary, &["branch", "--list"])
        .output()
        .expect("spawn git branch");
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    for line in s.lines() {
        // git prefixes lines with '*' (current), '+' (checked out
        // in another worktree), or ' ' (other) followed by a space.
        let name = line.trim_start_matches(['*', '+', ' ']).trim();
        if name.len() == 25 && name.chars().filter(|c| *c == '-').count() == 1 {
            return Some(name.to_owned());
        }
    }
    None
}

pub fn poll_for_path(path: &Path, deadline: Duration) {
    let until = Instant::now() + deadline;
    while !path.exists() {
        if Instant::now() >= until {
            panic!("timeout waiting for {}", path.display());
        }
        thread::sleep(Duration::from_millis(50));
    }
}

pub fn spawn_prompt(dest: &Path, harness: &Path, user_message: &str) -> Child {
    Command::new(lernie_bin())
        .arg("prompt")
        .arg(dest)
        .arg(user_message)
        .env("PATH", path_env_with_adapter())
        .env("ANTHROPIC_API_KEY", "test-key")
        .env("LERNIE_HOME", harness)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn lernie prompt")
}

/// Anthropic-native SSE that resolves to one happy `message_stop`.
/// Used as the eventual-completion body for delayed-mock scenarios.
pub const HAPPY_SSE: &str = concat!(
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
