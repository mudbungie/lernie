//! Smoke test for the §3.5 UI contract: the public view-model API is
//! pure over filesystem state, so multiple frontends running against
//! one repo cannot corrupt each other. The "second frontend" here is
//! just a sibling thread that calls the same public surface a real
//! `lernie-ui-web` would.
//!
//! Mechanism: build a minimal v0.3.1-shape conv-repo on disk, then
//! derive `GitTree::from_repo` from N threads simultaneously and
//! assert they all observe identical state. The fixture itself stays
//! frozen for the duration of the read window — the test is about
//! reentrancy of the read path, not about race-tolerance under writes.

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

/// Build a minimal v0.3.1 conv-repo: `<dir>/root/` is the primary
/// worktree on `main`, with one merged conversation `c001` and one
/// in-flight conversation `c002`. ARCH §2.2 / §2.3 layout.
fn fixture() -> TempDir {
    let dir = TempDir::new().unwrap();
    let primary = dir.path().join("root");
    std::fs::create_dir_all(&primary).unwrap();
    run_git(&primary, &["init", "-q", "-b", "main"]);
    run_git(&primary, &["config", "user.email", "t@t.local"]);
    run_git(&primary, &["config", "user.name", "Tester"]);
    run_git(&primary, &["config", "commit.gpgsign", "false"]);
    std::fs::write(primary.join("README"), "x").unwrap();
    run_git(&primary, &["add", "README"]);
    run_git(&primary, &["commit", "-q", "-m", "init"]);
    run_git(&primary, &["checkout", "-q", "-b", "c001"]);
    std::fs::write(primary.join("goal.md"), "g").unwrap();
    run_git(&primary, &["add", "goal.md"]);
    run_git(&primary, &["commit", "-q", "-m", "dispatch c001"]);
    run_git(&primary, &["checkout", "-q", "main"]);
    run_git(&primary, &["merge", "--no-ff", "-q", "--no-edit", "c001"]);
    run_git(&primary, &["checkout", "-q", "-b", "c002"]);
    std::fs::write(primary.join("goal.md"), "g2").unwrap();
    run_git(&primary, &["add", "goal.md"]);
    run_git(&primary, &["commit", "-q", "-m", "dispatch c002"]);
    run_git(&primary, &["checkout", "-q", "main"]);
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
