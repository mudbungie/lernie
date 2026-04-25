//! Shared test fixture: a tempdir-backed git repo plus helpers for
//! committing v0.1-shape and v0.2-shape exchanges.
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
    pub(super) path: PathBuf,
}

impl Fixture {
    pub(super) fn new() -> Self {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();
        run_git(&path, &["init", "-q", "-b", "main"]);
        run_git(&path, &["config", "user.email", "t@t.local"]);
        run_git(&path, &["config", "user.name", "Tester"]);
        run_git(&path, &["config", "commit.gpgsign", "false"]);
        fs::create_dir(path.join("exchanges")).unwrap();
        Self { _dir: dir, path }
    }

    pub(super) fn commit_v01_exchange(&self, id: &str, user_message: &str) {
        let rel = format!("exchanges/{id}.json");
        let json = format!(
            r#"{{"user_message":"{}"}}"#,
            user_message.replace('"', "\\\"")
        );
        fs::write(self.path.join(&rel), json).unwrap();
        run_git(&self.path, &["add", &rel]);
        run_git(&self.path, &["commit", "-q", "-m", &format!("ex {id}")]);
    }

    pub(super) fn commit_other(&self, file: &str, body: &str) {
        fs::write(self.path.join(file), body).unwrap();
        run_git(&self.path, &["add", file]);
        run_git(&self.path, &["commit", "-q", "-m", &format!("add {file}")]);
    }

    pub(super) fn commit_malformed_v01_exchange(&self, id: &str) {
        let rel = format!("exchanges/{id}.json");
        fs::write(self.path.join(&rel), "{ not valid json").unwrap();
        run_git(&self.path, &["add", &rel]);
        run_git(&self.path, &["commit", "-q", "-m", &format!("bad {id}")]);
    }

    pub(super) fn commit_v02_merged_exchange(&self, id: &str, user_message: &str) {
        self.build_v02_branch(id, user_message);
        let branch = format!("ex/{id}");
        run_git(&self.path, &["checkout", "-q", "main"]);
        run_git(
            &self.path,
            &[
                "merge",
                "--no-ff",
                "-q",
                "-m",
                &format!("merge {branch}"),
                &branch,
            ],
        );
    }

    pub(super) fn build_v02_in_flight(&self, id: &str, user_message: &str) {
        self.build_v02_branch(id, user_message);
        run_git(&self.path, &["checkout", "-q", "main"]);
    }

    fn build_v02_branch(&self, id: &str, user_message: &str) {
        let branch = format!("ex/{id}");
        run_git(&self.path, &["checkout", "-q", "-b", &branch, "main"]);
        let step_dir = format!("exchanges/{id}/steps/001");
        fs::create_dir_all(self.path.join(&step_dir)).unwrap();
        let request_json = serde_json::json!({
            "model": "m",
            "messages": [{"role": "user", "content": user_message}],
        });
        fs::write(
            self.path.join(format!("{step_dir}/request.json")),
            serde_json::to_vec_pretty(&request_json).unwrap(),
        )
        .unwrap();
        run_git(&self.path, &["add", &format!("{step_dir}/request.json")]);
        run_git(
            &self.path,
            &[
                "commit",
                "-q",
                "-m",
                &format!("step 001: dispatch [ex {id}]"),
            ],
        );
        let response_json = serde_json::json!({
            "content": [{"type": "text", "text": "pong"}],
            "stop_reason": "end_turn"
        });
        fs::write(
            self.path.join(format!("{step_dir}/response.json")),
            serde_json::to_vec_pretty(&response_json).unwrap(),
        )
        .unwrap();
        run_git(&self.path, &["add", &format!("{step_dir}/response.json")]);
        run_git(
            &self.path,
            &[
                "commit",
                "-q",
                "-m",
                &format!("step 001: response [ex {id}]"),
            ],
        );
        // Compactor sub-branch: one commit on inv/<id>/c, merged back
        // --no-ff into the exchange branch. Mirrors the real shape.
        let cmp_branch = format!("inv/{id}/c");
        run_git(&self.path, &["checkout", "-q", "-b", &cmp_branch, &branch]);
        fs::create_dir_all(self.path.join(".agent/compactions")).unwrap();
        fs::write(
            self.path.join(".agent/compactions/001.md"),
            format!("exchange {id}: pong\n"),
        )
        .unwrap();
        run_git(&self.path, &["add", ".agent/compactions/001.md"]);
        run_git(
            &self.path,
            &[
                "commit",
                "-q",
                "-m",
                &format!("compaction: terminal summary [ex {id}]"),
            ],
        );
        run_git(&self.path, &["checkout", "-q", &branch]);
        run_git(
            &self.path,
            &[
                "merge",
                "--no-ff",
                "-q",
                "-m",
                &format!("compactor merge [ex {id}]"),
                &cmp_branch,
            ],
        );
        run_git(&self.path, &["branch", "-q", "-D", &cmp_branch]);
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
