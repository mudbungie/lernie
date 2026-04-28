//! Shared test fixture: a tempdir-backed conv-repo (ARCH §2.2 layout)
//! plus helpers for committing v0.3.1 conversations and dispatching
//! in-flight subagent branches.
//!
//! Tests hit a real git binary rather than mocking; fixtures are cheap
//! to spin up and the renderer's contract is explicitly with the CLI,
//! so mocking would mean testing our mock.
//!
//! Post-bl-c22c, the fixture follows the relocated step-record shape
//! (ARCH §2.3): step records (`request.json` / `response.json`) live at
//! `<conv-repo>/steps/<conv-id>/<NNN>/`, *outside* every worktree, and
//! are not committed to git. The conversation branch carries a
//! dispatch commit (`goal.md` + `soul.md`) and a compactor merge — the
//! shape produced by `src/prompt/dispatch` + `src/prompt/compactor`.
//! Conversation detection on the trunk keys off the default
//! `Merge branch '<conv-id>'` subject, written by `git merge --no-ff`
//! with no `-m` (mirroring `src/prompt/merge::rebase_and_merge`).

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
    /// Conv-repo root — the dir holding `root/`, control files
    /// (`steps/`, `souls/`, etc.), and any subagent worktrees. This is
    /// what `GitTree::from_repo` is passed in production (ARCH §2.2).
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

    /// Build a v0.3.1-shape conversation branch and merge it back to
    /// `main` with `--no-ff`. Branch carries a dispatch commit plus a
    /// compactor sub-branch merge; the user-message step record lands
    /// on disk at `<conv-repo>/steps/<conv-id>/001/request.json`,
    /// outside every worktree (ARCH §2.3).
    pub(super) fn commit_v03_merged_conversation(&self, conv_id: &str, user_message: &str) {
        self.build_v03_branch(conv_id, user_message);
        run_git(&self.primary, &["checkout", "-q", "main"]);
        // `git merge --no-ff <branch>` with no `-m` writes the default
        // subject `Merge branch '<branch>'` — what `parse_merge_subject`
        // keys off (ARCH §2.3, src/prompt/merge::rebase_and_merge).
        run_git(
            &self.primary,
            &["merge", "--no-ff", "-q", "--no-edit", conv_id],
        );
    }

    /// Build a v0.3.1-shape conversation branch and leave it unmerged
    /// (the in-flight case the renderer surfaces under "in-flight
    /// conversations").
    pub(super) fn build_v03_in_flight(&self, conv_id: &str, user_message: &str) {
        self.build_v03_branch(conv_id, user_message);
        run_git(&self.primary, &["checkout", "-q", "main"]);
    }

    /// Write the diagnostic step record for `conv_id`'s first step at
    /// the conv-repo root (ARCH §2.3). Called by `build_v03_branch`,
    /// also exposed for tests that need to seed a step record without
    /// building a full branch.
    pub(super) fn write_step_record(&self, conv_id: &str, user_message: &str) {
        let step_dir = self.path.join("steps").join(conv_id).join("001");
        fs::create_dir_all(&step_dir).unwrap();
        let request_json = serde_json::json!({
            "model": "m",
            "messages": [{"role": "user", "content": user_message}],
        });
        fs::write(
            step_dir.join("request.json"),
            serde_json::to_vec_pretty(&request_json).unwrap(),
        )
        .unwrap();
    }

    /// Write a partial `response.json` for `conv_id`'s `seq`-th step.
    /// Each `event` is a JSONL line (no trailing newline); they are
    /// joined with `\n` and a trailing `\n` is appended, mirroring the
    /// shape `src/prompt/dispatch::stream::run_complete` produces line
    /// by line. The fd closes when this helper returns — the harness's
    /// IN_CLOSE_WRITE semantics aren't reproduced here, but the on-disk
    /// snapshot the UI tails is identical, which is what the
    /// stateless-re-read view-model contract (ARCH §3.5) cares about.
    pub(super) fn write_response_events(&self, conv_id: &str, seq: u32, events: &[&str]) {
        let step_dir = self
            .path
            .join("steps")
            .join(conv_id)
            .join(format!("{seq:03}"));
        fs::create_dir_all(&step_dir).unwrap();
        let mut payload = events.join("\n");
        if !events.is_empty() {
            payload.push('\n');
        }
        fs::write(step_dir.join("response.json"), payload).unwrap();
    }

    fn build_v03_branch(&self, conv_id: &str, user_message: &str) {
        run_git(&self.primary, &["checkout", "-q", "-b", conv_id, "main"]);
        // Dispatch commit (ARCH §2.3 step 2): goal.md + soul.md only.
        // The user message is *not* in the tree — it lives in
        // `request.json` on disk under `<conv-repo>/steps/...`.
        fs::write(self.primary.join("goal.md"), user_message).unwrap();
        fs::write(self.primary.join("soul.md"), "you are a tester\n").unwrap();
        run_git(&self.primary, &["add", "goal.md", "soul.md"]);
        run_git(
            &self.primary,
            &["commit", "-q", "-m", &format!("dispatch [{conv_id}]")],
        );
        // Compactor sub-branch: hyphenated descent off the conversation
        // branch (ARCH §2.3); one summary commit, merged --no-ff back
        // into the conversation branch with the default merge subject.
        // Mirrors the real shape produced by `dispatch_compactor` plus
        // `src/prompt/merge::rebase_and_merge`.
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
            &["merge", "--no-ff", "-q", "--no-edit", &cmp_branch],
        );
        run_git(&self.primary, &["branch", "-q", "-D", &cmp_branch]);
        // Step record on disk, outside every worktree (ARCH §2.3).
        self.write_step_record(conv_id, user_message);
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
