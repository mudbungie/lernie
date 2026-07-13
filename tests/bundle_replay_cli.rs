//! End-to-end subprocess tests for `lernie bundle` and `lernie replay`
//! (ARCH §9.2 *Replay and archival*). These drive the real `git`
//! transport — bundle create, fetch-from-bundle, worktree materialize —
//! against a hand-built workspace, so the argument shapes the unit tests
//! stub out are proven correct on a real repo.

use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

fn lernie_bin() -> &'static str {
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

fn git(dest: &Path, args: &[&str]) {
    let mut cmd = Command::new("git");
    for var in INHERITED_GIT_ENV {
        cmd.env_remove(var);
    }
    let out = cmd.arg("-C").arg(dest).args(args).output().expect("git");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

const PARENT: &str = "20260101-p1";
const CHILD: &str = "20260101-p1-20260102-c1";
/// A sibling root agent that must NOT be captured by the subtree bundle.
const UNRELATED: &str = "20260101-z9";

/// Build a workspace with a parent agent branch carrying a child branch,
/// an unrelated root branch, and diagnostic slices for the subtree.
fn workspace() -> TempDir {
    let ws = TempDir::new().unwrap();
    let root = ws.path().join("root");
    std::fs::create_dir_all(&root).unwrap();
    git(&root, &["init", "-q", "-b", "main"]);
    git(&root, &["config", "user.email", "t@test.invalid"]);
    git(&root, &["config", "user.name", "t"]);
    git(&root, &["commit", "-q", "--allow-empty", "-m", "init"]);

    // Parent agent branch + its worktree, with a goal work product.
    let parent_wt = ws.path().join(PARENT);
    git(
        &root,
        &[
            "worktree",
            "add",
            "-q",
            parent_wt.to_str().unwrap(),
            "-b",
            PARENT,
        ],
    );
    std::fs::write(parent_wt.join("goal.md"), "parent goal\n").unwrap();
    git(&parent_wt, &["add", "-A"]);
    git(&parent_wt, &["commit", "-q", "-m", "parent goal"]);

    // Child branch forked off the parent tip.
    let child_wt = ws.path().join(CHILD);
    git(
        &parent_wt,
        &[
            "worktree",
            "add",
            "-q",
            child_wt.to_str().unwrap(),
            "-b",
            CHILD,
        ],
    );
    std::fs::write(child_wt.join("goal.md"), "child goal\n").unwrap();
    git(&child_wt, &["add", "-A"]);
    git(&child_wt, &["commit", "-q", "-m", "child goal"]);

    git(&root, &["branch", UNRELATED]);

    // Diagnostic slices (§2.2): steps for both agents + an inbox message.
    let steps = ws.path().join("steps").join(PARENT).join("001");
    std::fs::create_dir_all(&steps).unwrap();
    std::fs::write(steps.join("meta.json"), "{\"commit\":\"x\"}").unwrap();
    let inbox = ws.path().join("inbox").join(PARENT);
    std::fs::create_dir_all(&inbox).unwrap();
    std::fs::write(inbox.join("user-001.md"), "a message\n").unwrap();
    ws
}

fn run(args: &[&str], home: Option<&Path>) -> std::process::Output {
    let mut cmd = Command::new(lernie_bin());
    for var in INHERITED_GIT_ENV {
        cmd.env_remove(var);
    }
    if let Some(h) = home {
        cmd.env("LERNIE_HOME", h);
    }
    cmd.args(args).output().expect("lernie")
}

#[test]
fn bundle_then_replay_round_trips_the_subtree() {
    let ws = workspace();
    let archive = TempDir::new().unwrap();
    let arch_dir = archive.path().join("arch");

    let out = run(
        &[
            "bundle",
            ws.path().to_str().unwrap(),
            PARENT,
            arch_dir.to_str().unwrap(),
        ],
        None,
    );
    assert!(
        out.status.success(),
        "bundle: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // One bundle plus two slices (§9.2).
    assert!(arch_dir.join("agents.bundle").exists());
    assert!(
        arch_dir
            .join("steps")
            .join(PARENT)
            .join("001/meta.json")
            .exists()
    );
    assert!(
        arch_dir
            .join("inbox")
            .join(PARENT)
            .join("user-001.md")
            .exists()
    );

    // The bundle carries the subtree, and only the subtree.
    let heads = Command::new("git")
        .args([
            "bundle",
            "list-heads",
            arch_dir.join("agents.bundle").to_str().unwrap(),
        ])
        .output()
        .expect("list-heads");
    let heads = String::from_utf8_lossy(&heads.stdout);
    assert!(heads.contains(PARENT), "heads: {heads}");
    assert!(heads.contains(CHILD), "heads: {heads}");
    assert!(
        !heads.contains(UNRELATED),
        "unrelated branch leaked: {heads}"
    );

    // Replay into an isolated scratch home.
    let home = TempDir::new().unwrap();
    let out = run(&["replay", arch_dir.to_str().unwrap()], Some(home.path()));
    assert!(
        out.status.success(),
        "replay: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let scratch = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim().to_string());
    assert_eq!(scratch, home.path().join("replays").join(PARENT));

    // The reconstructed repo has both branches; the primary worktree is
    // materialized; the slices are restored.
    let scratch_root = scratch.join("root");
    let branches = Command::new("git")
        .arg("-C")
        .arg(&scratch_root)
        .args(["branch", "--list", "--format=%(refname:short)"])
        .output()
        .expect("branch");
    let branches = String::from_utf8_lossy(&branches.stdout);
    assert!(
        branches.contains(PARENT) && branches.contains(CHILD),
        "branches: {branches}"
    );
    assert_eq!(
        std::fs::read_to_string(scratch.join(PARENT).join("goal.md")).unwrap(),
        "parent goal\n"
    );
    assert!(
        scratch
            .join("steps")
            .join(PARENT)
            .join("001/meta.json")
            .exists()
    );
    assert!(
        scratch
            .join("inbox")
            .join(PARENT)
            .join("user-001.md")
            .exists()
    );
}

#[test]
fn bundle_rejects_unknown_agent() {
    let ws = workspace();
    let archive = TempDir::new().unwrap();
    let out = run(
        &[
            "bundle",
            ws.path().to_str().unwrap(),
            "20260101-nope",
            archive.path().join("a").to_str().unwrap(),
        ],
        None,
    );
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("no branch matches"));
}

#[test]
fn replay_rejects_missing_bundle() {
    let empty = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let out = run(
        &["replay", empty.path().to_str().unwrap()],
        Some(home.path()),
    );
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("not found"));
}

/// In-process coverage of `archive::replay_cli` (the lib wiring the bin
/// delegates to): it resolves the scratch base under `LERNIE_HOME`'s data
/// root and lands the scratch workspace there. Env is scoped per call and
/// `--test-threads=1` (tarpaulin.toml) keeps it serial.
#[test]
fn replay_cli_lands_under_lernie_home() {
    let ws = workspace();
    let archive = TempDir::new().unwrap();
    let arch_dir = archive.path().join("arch");
    lernie::archive::bundle(
        ws.path(),
        PARENT,
        &arch_dir,
        &lernie::template::RealGit::new(),
    )
    .expect("bundle");

    let home = TempDir::new().unwrap();
    // SAFETY: single-threaded test run (tarpaulin.toml `--test-threads=1`).
    unsafe { std::env::set_var("LERNIE_HOME", home.path()) };
    let scratch = lernie::archive::replay_cli(&arch_dir).expect("replay_cli");
    unsafe { std::env::remove_var("LERNIE_HOME") };

    assert_eq!(scratch, home.path().join("replays").join(PARENT));
    assert!(scratch.join("root").join(".git").exists());
    assert!(scratch.join(PARENT).join("goal.md").exists());
}
