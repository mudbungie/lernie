//! Smoke test for the §3.5 UI contract: the public view-model API is
//! pure over filesystem state, so multiple frontends running against
//! one repo cannot corrupt each other. The "second frontend" here is
//! just a sibling thread that calls the same public surface a real
//! `lernie-ui-web` would.
//!
//! Mechanism: build a minimal workspace (ARCH §2.2: bare repo.git,
//! config/default, agents/* refs) on disk, then derive
//! `GitTree::from_repo` from N threads simultaneously and assert they
//! all observe identical state. The fixture itself stays frozen for
//! the duration of the read window — the test is about reentrancy of
//! the read path, not about race-tolerance under writes.

use lernie_ui_egui::git_tree::GitTree;
use std::process::Command;
use std::sync::{Arc, Barrier};
use std::thread;
use tempfile::TempDir;

/// Env vars that, when inherited from the cargo-test process, override
/// `-C <repo>` and silently redirect every `git` invocation back to the
/// outer repo — the same scrub list `lernie-ui-egui::git_tree::cmd`
/// applies in production. Without this, fixture commits land on
/// whatever branch the tarpaulin/precommit context had checked out.
const INHERITED_GIT_ENV: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_PREFIX",
    "GIT_COMMON_DIR",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
];

fn run_git(repo: &std::path::Path, args: &[&str]) {
    let mut cmd = Command::new("git");
    for var in INHERITED_GIT_ENV {
        cmd.env_remove(var);
    }
    let status = cmd.arg("-C").arg(repo).args(args).status().unwrap();
    assert!(status.success(), "git {args:?}");
}

/// Build a minimal workspace: a bare `repo.git` with `config/default`
/// (one config commit) and two agent branches `agents/c-001` and
/// `agents/c-002`, each with a dispatch commit. ARCH §2.2 / §2.3.
fn fixture() -> TempDir {
    let dir = TempDir::new().unwrap();
    let repo = dir.path().join("repo.git");
    std::fs::create_dir_all(&repo).unwrap();
    run_git(&repo, &["init", "-q", "--bare", "-b", "config/default"]);
    run_git(&repo, &["config", "user.email", "t@t.local"]);
    run_git(&repo, &["config", "user.name", "Tester"]);
    run_git(&repo, &["config", "commit.gpgsign", "false"]);
    let author = dir.path().join(".author");
    let author_str = author.to_string_lossy().to_string();
    run_git(
        &repo,
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
    std::fs::write(author.join("version"), "1\n").unwrap();
    run_git(&author, &["add", "version"]);
    run_git(&author, &["commit", "-q", "-m", "config: init"]);
    run_git(&repo, &["worktree", "remove", author_str.as_str()]);
    for id in ["c-001", "c-002"] {
        let wt = dir.path().join("agents").join(id);
        let wt_str = wt.to_string_lossy().to_string();
        let branch = format!("agents/{id}");
        run_git(
            &repo,
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
        std::fs::write(wt.join("goal.md"), "g").unwrap();
        run_git(&wt, &["add", "goal.md"]);
        run_git(&wt, &["commit", "-q", "-m", &format!("dispatch [{id}]")]);
    }
    dir
}

/// N concurrent `GitTree::from_repo` calls against one frozen fixture
/// observe the same state — demonstrates the §3.5 reentrancy claim
/// ("two frontends running against one repo cannot corrupt each other
/// because neither writes repo state; both observe the same on-disk
/// ground truth").
#[test]
fn parallel_frontends_observe_identical_view_model() {
    let dir = fixture();
    let repo = dir.path().to_path_buf();
    const N: usize = 4;
    let barrier = Arc::new(Barrier::new(N));
    let handles: Vec<_> = (0..N)
        .map(|_| {
            let repo = repo.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                barrier.wait();
                GitTree::from_repo(&repo).unwrap()
            })
        })
        .collect();
    let trees: Vec<GitTree> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    let first = &trees[0];
    for (i, t) in trees.iter().enumerate().skip(1) {
        assert_eq!(
            t, first,
            "frontend {i}'s view-model diverged from frontend 0"
        );
    }
}
