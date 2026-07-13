//! Shared test fixture: a tempdir-backed workspace (ARCH §2.2 layout —
//! bare `repo.git`, `config/default`, `agents/*` refs) plus helpers for
//! amending the config lineage and dispatching agent branches.
//!
//! Tests hit a real git binary rather than mocking; fixtures are cheap
//! to spin up and the renderer's contract is explicitly with the CLI,
//! so mocking would mean testing our mock.
//!
//! Step records (`request.json` / `response.json`) live at
//! `<workspace>/steps/<agent-id>/<NNN>/`, *outside* every worktree, and
//! are not committed to git (ARCH §2.3). An agent branch carries a
//! dispatch commit (`goal.md` + `soul.md`) and a compaction merge — the
//! shape produced by `src/prompt/dispatch` + `src/prompt/compactor`.

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
    /// Workspace root — the dir holding `repo.git/`, `steps/`,
    /// `inbox/`, and the `agents/` worktrees. This is what
    /// `GitTree::from_repo` is passed in production (ARCH §2.2).
    pub(super) path: PathBuf,
    /// The bare workspace repository (`<workspace>/repo.git`). All
    /// ref-level git commands run against it, mirroring the harness.
    pub(super) repo: PathBuf,
}

impl Fixture {
    pub(super) fn new() -> Self {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();
        let repo = path.join("repo.git");
        fs::create_dir_all(&repo).unwrap();
        run_git(&repo, &["init", "-q", "--bare", "-b", "config/default"]);
        run_git(&repo, &["config", "user.email", "t@t.local"]);
        run_git(&repo, &["config", "user.name", "Tester"]);
        run_git(&repo, &["config", "commit.gpgsign", "false"]);
        let fx = Self {
            _dir: dir,
            path,
            repo,
        };
        // The first config commit (orphan root, §2.2).
        let author = fx.path.join(".author");
        let author_str = author.to_string_lossy().to_string();
        run_git(
            &fx.repo,
            &[
                "worktree",
                "add",
                "-q",
                "--orphan",
                "-b",
                "config/default",
                author_str.as_str(),
            ],
        );
        fs::write(author.join("version"), "1\n").unwrap();
        run_git(&author, &["add", "version"]);
        run_git(
            &author,
            &["commit", "-q", "-m", "config: init [config/default]"],
        );
        run_git(&fx.repo, &["worktree", "remove", author_str.as_str()]);
        fx
    }

    /// Advance the config lineage with one edit (§2.3 branch
    /// advancement: only user config edits move a config branch).
    pub(super) fn commit_other(&self, file: &str, body: &str) {
        let author = self.path.join(".amend");
        let author_str = author.to_string_lossy().to_string();
        run_git(
            &self.repo,
            &[
                "worktree",
                "add",
                "-q",
                author_str.as_str(),
                "config/default",
            ],
        );
        fs::write(author.join(file), body).unwrap();
        run_git(&author, &["add", file]);
        run_git(&author, &["commit", "-q", "-m", &format!("add {file}")]);
        run_git(&self.repo, &["worktree", "remove", author_str.as_str()]);
    }

    /// Build an agent branch `agents/<conv_id>` with its worktree at
    /// `agents/<conv_id>/`: a dispatch commit plus a compaction merge
    /// (the one merge, §2.6); the user-message step record lands on
    /// disk at `<workspace>/steps/<conv-id>/001/request.json`, outside
    /// every worktree (ARCH §2.3).
    pub(super) fn build_agent(&self, conv_id: &str, user_message: &str) {
        let wt = self.path.join("agents").join(conv_id);
        let wt_str = wt.to_string_lossy().to_string();
        let branch = format!("agents/{conv_id}");
        run_git(
            &self.repo,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                branch.as_str(),
                wt_str.as_str(),
                "config/default",
            ],
        );
        // Dispatch commit (ARCH §2.3 step 2): goal.md + soul.md; the
        // control files leave the tree (only `version` here).
        fs::write(wt.join("goal.md"), user_message).unwrap();
        fs::write(wt.join("soul.md"), "you are a tester\n").unwrap();
        run_git(&wt, &["rm", "-q", "--ignore-unmatch", "version"]);
        run_git(&wt, &["add", "goal.md", "soul.md"]);
        run_git(
            &wt,
            &["commit", "-q", "-m", &format!("dispatch [{conv_id}]")],
        );
        // Compactor child: hyphenated descent off the agent branch
        // (ARCH §2.3); one summary commit, merged --no-ff back into the
        // agent branch — the compaction merge (§2.6).
        let cmp_id = format!("{conv_id}-c");
        let cmp_branch = format!("agents/{cmp_id}");
        let cmp_wt = self.path.join("agents").join(&cmp_id);
        let cmp_str = cmp_wt.to_string_lossy().to_string();
        run_git(
            &self.repo,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                cmp_branch.as_str(),
                cmp_str.as_str(),
                branch.as_str(),
            ],
        );
        fs::create_dir_all(cmp_wt.join("summary")).unwrap();
        fs::write(
            cmp_wt.join("summary/001.md"),
            format!("conversation {conv_id}: pong\n"),
        )
        .unwrap();
        run_git(&cmp_wt, &["add", "summary/001.md"]);
        run_git(
            &cmp_wt,
            &[
                "commit",
                "-q",
                "-m",
                &format!("compaction: terminal summary [{conv_id}]"),
            ],
        );
        run_git(&wt, &["merge", "--no-ff", "-q", "--no-edit", &cmp_branch]);
        run_git(&self.repo, &["worktree", "remove", cmp_str.as_str()]);
        run_git(&self.repo, &["branch", "-q", "-D", &cmp_branch]);
        // Step record on disk, outside every worktree (ARCH §2.3).
        self.write_step_record(conv_id, user_message);
    }

    /// Write the diagnostic step record for `conv_id`'s first step at
    /// the workspace root (ARCH §2.3). Called by `build_agent`, also
    /// exposed for tests that need to seed a step record without
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

    /// Write a tool-call `input.json` (and optionally `output.json`)
    /// under `<workspace>/steps/<conv-id>/<seq>/tools/<tool_id>/`. Mirrors
    /// the executor's on-disk shape (ARCH §3.3): `input.json` lands first
    /// at dispatch, `output.json` only after the tool exits. Pass `None`
    /// for the in-flight case.
    pub(super) fn write_tool_call(
        &self,
        conv_id: &str,
        seq: u32,
        tool_id: &str,
        output: Option<&[u8]>,
    ) {
        let tool_dir = self
            .path
            .join(format!("steps/{conv_id}/{seq:03}/tools/{tool_id}",));
        fs::create_dir_all(&tool_dir).unwrap();
        fs::write(tool_dir.join("input.json"), b"{}").unwrap();
        if let Some(out) = output {
            fs::write(tool_dir.join("output.json"), out).unwrap();
        }
    }

    /// Deposit a pending message into `agent_id`'s inbox at
    /// `<workspace>/inbox/<agent-id>/<filename>` (ARCH §2.11). Mirrors a
    /// `<sender>-<NNN>.md` deposit the frontend counts for the
    /// pending-message indicator (§7.1).
    pub(super) fn deposit_message(&self, agent_id: &str, filename: &str, body: &str) {
        let inbox = self.path.join("inbox").join(agent_id);
        fs::create_dir_all(&inbox).unwrap();
        fs::write(inbox.join(filename), body).unwrap();
    }

    /// Point a `refs/lernie/<kind>/<agent-id>` mark ref at `config/default`
    /// (any commit is fine — the frontend reads existence, not content).
    /// Mirrors `transfer::decline` (§2.6) and `budget::mark_exhausted`
    /// (§6), which key the mark off the raw agent id.
    pub(super) fn mark_ref(&self, refname: &str) {
        run_git(&self.repo, &["update-ref", refname, "config/default"]);
    }

    /// Build a child agent branch `agents/<child_id>` off `parent_id`'s
    /// branch tip — a hyphenated-descent fork (§2.3). Used to exercise the
    /// descent-tree render (§7.1). The child carries one dispatch commit.
    pub(super) fn build_child(&self, parent_id: &str, child_id: &str) {
        let wt = self.path.join("agents").join(child_id);
        let wt_str = wt.to_string_lossy().to_string();
        let branch = format!("agents/{child_id}");
        run_git(
            &self.repo,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                branch.as_str(),
                wt_str.as_str(),
                &format!("agents/{parent_id}"),
            ],
        );
        fs::write(wt.join("child.md"), "child work\n").unwrap();
        run_git(&wt, &["add", "child.md"]);
        run_git(
            &wt,
            &["commit", "-q", "-m", &format!("dispatch [{child_id}]")],
        );
        run_git(&self.repo, &["worktree", "remove", wt_str.as_str()]);
    }

    /// Write a partial `response.json` for `conv_id`'s `seq`-th step.
    /// Each `event` is a JSONL line (no trailing newline); they are
    /// joined with `\n` and a trailing `\n` is appended, mirroring the
    /// shape the executor produces line by line. The fd closes when
    /// this helper returns — the harness's IN_CLOSE_WRITE semantics
    /// aren't reproduced here, but the on-disk snapshot the UI tails is
    /// identical, which is what the stateless-re-read view-model
    /// contract (ARCH §3.5) cares about.
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
