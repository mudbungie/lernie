//! Shared test fixture: a tempdir-backed conv-repo (ARCH §2.2 layout)
//! plus helpers for committing v0.3-shape conversations and dispatching
//! in-flight subagent branches.
//!
//! Tests hit a real git binary rather than mocking; fixtures are cheap
//! to spin up and the renderer's contract is explicitly with the CLI,
//! so mocking would mean testing our mock.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::{TempDir, tempdir};

/// Env vars to scrub on every `git` spawn (mirrors `cmd.rs`).
const INHERITED_GIT_ENV: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_PREFIX",
    "GIT_COMMON_DIR",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
];

pub(super) struct Fixture {
    _dir: TempDir,
    /// Conv-repo root — the dir holding `root/`, control files, and
    /// any subagent worktrees. This is what `GitTree::from_repo` is
    /// passed in production (ARCH §2.2).
    pub(super) path: PathBuf,
    /// Primary worktree path (`<conv-repo>/root/`). All git commands
    /// in tests run from here, mirroring the harness.
    pub(super) primary: PathBuf,
}

impl Fixture {
    pub(super) fn new() -> Self {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();
        let primary = path.join("root");
        fs::create_dir_all(&primary).unwrap();
        run_git(&primary, &["init", "-q", "-b", "main"]);
        run_git(&primary, &["config", "user.email", "t@t.local"]);
        run_git(&primary, &["config", "user.name", "Tester"]);
        run_git(&primary, &["config", "commit.gpgsign", "false"]);
        Self {
            _dir: dir,
            path,
            primary,
        }
    }

    /// Land a non-conversation commit on `main` (e.g. README, scaffold
    /// tweak). Used to populate the trunk with commits the renderer
    /// must surface but not label as conversations.
    pub(super) fn commit_other(&self, file: &str, body: &str) {
        fs::write(self.primary.join(file), body).unwrap();
        run_git(&self.primary, &["add", file]);
        run_git(
            &self.primary,
            &["commit", "-q", "-m", &format!("add {file}")],
        );
    }

    /// Build a v0.3-shape conversation branch and merge it back to
    /// `main` with `--no-ff`, mirroring the dispatch + response +
    /// terminal-compaction sequence (ARCH §2.3, §2.6).
    pub(super) fn commit_v03_merged_conversation(&self, conv_id: &str, user_message: &str) {
        self.build_v03_branch(conv_id, user_message);
        run_git(&self.primary, &["checkout", "-q", "main"]);
        run_git(
            &self.primary,
            &[
                "merge",
                "--no-ff",
                "-q",
                "-m",
                &format!("merge {conv_id}"),
                conv_id,
            ],
        );
    }

    /// Build a v0.3-shape conversation branch and leave it unmerged
    /// (the in-flight case the renderer surfaces under "in-flight
    /// conversations").
    pub(super) fn build_v03_in_flight(&self, conv_id: &str, user_message: &str) {
        self.build_v03_branch(conv_id, user_message);
        run_git(&self.primary, &["checkout", "-q", "main"]);
    }

    fn build_v03_branch(&self, conv_id: &str, user_message: &str) {
        run_git(&self.primary, &["checkout", "-q", "-b", conv_id, "main"]);
        let step_dir = format!("steps/{conv_id}/001");
        fs::create_dir_all(self.primary.join(&step_dir)).unwrap();
        let request_json = serde_json::json!({
            "model": "m",
            "messages": [{"role": "user", "content": user_message}],
        });
        fs::write(
            self.primary.join(format!("{step_dir}/request.json")),
            serde_json::to_vec_pretty(&request_json).unwrap(),
        )
        .unwrap();
        run_git(&self.primary, &["add", &format!("{step_dir}/request.json")]);
        run_git(
            &self.primary,
            &[
                "commit",
                "-q",
                "-m",
                &format!("step 001: dispatch [{conv_id}]"),
            ],
        );
        let response_json = serde_json::json!({
            "content": [{"type": "text", "text": "pong"}],
            "stop_reason": "end_turn"
        });
        fs::write(
            self.primary.join(format!("{step_dir}/response.json")),
            serde_json::to_vec_pretty(&response_json).unwrap(),
        )
        .unwrap();
        run_git(
            &self.primary,
            &["add", &format!("{step_dir}/response.json")],
        );
        run_git(
            &self.primary,
            &[
                "commit",
                "-q",
                "-m",
                &format!("step 001: response [{conv_id}]"),
            ],
        );
        // Compactor sub-branch: hyphenated descent off the conversation
        // branch (ARCH §2.3); one summary commit, merged --no-ff back
        // into the conversation branch. Mirrors the real shape produced
        // by `dispatch_compactor`.
        let cmp_branch = format!("{conv_id}-c");
        run_git(
            &self.primary,
            &["checkout", "-q", "-b", &cmp_branch, conv_id],
        );
        fs::create_dir_all(self.primary.join("summary")).unwrap();
        fs::write(
            self.primary.join("summary/001.md"),
            format!("conversation {conv_id}: pong\n"),
        )
        .unwrap();
        run_git(&self.primary, &["add", "summary/001.md"]);
        run_git(
            &self.primary,
            &[
                "commit",
                "-q",
                "-m",
                &format!("compaction: terminal summary [{conv_id}]"),
            ],
        );
        run_git(&self.primary, &["checkout", "-q", conv_id]);
        run_git(
            &self.primary,
            &[
                "merge",
                "--no-ff",
                "-q",
                "-m",
                &format!("compactor merge [{conv_id}]"),
                &cmp_branch,
            ],
        );
        run_git(&self.primary, &["branch", "-q", "-D", &cmp_branch]);
    }
}

pub(super) fn run_git(repo: &Path, args: &[&str]) {
    let mut cmd = Command::new("git");
    for var in INHERITED_GIT_ENV {
        cmd.env_remove(var);
    }
    let status = cmd
        .arg("-C")
        .arg(repo)
        .args(args)
        .env("GIT_AUTHOR_DATE", "2026-04-22T12:00:00Z")
        .env("GIT_COMMITTER_DATE", "2026-04-22T12:00:00Z")
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?} failed");
}
