//! End-to-end subprocess tests for the operator verb `lernie scan`
//! (ARCH §2.11 *Crashes are a failure class*, §8) and for its removal
//! from the driver hot paths: `lernie prompt` and `lernie dispatch`
//! never run the workspace-wide sweep.

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

/// A root-shaped agent id and one of its children (§2.3 token shape).
const PARENT: &str = "20260101-p1";
const CHILD: &str = "20260101-p1-20260102-c1";

/// A workspace whose git state shows a hard-crashed child: real `root`
/// worktree on `main`, a parent branch, and a child branch that never
/// deposited a result — the §8 sweep's candidate.
fn workspace_with_crashed_child() -> TempDir {
    let ws = TempDir::new().unwrap();
    let root = ws.path().join("root");
    std::fs::create_dir_all(&root).unwrap();
    git(&root, &["init", "-b", "main"]);
    git(&root, &["config", "user.email", "t@test.invalid"]);
    git(&root, &["config", "user.name", "t"]);
    git(&root, &["commit", "--allow-empty", "-m", "init"]);
    git(&root, &["branch", PARENT]);
    git(&root, &["branch", CHILD]);
    ws
}

fn died_deposit(ws: &Path) -> PathBuf {
    ws.join("inbox")
        .join(PARENT)
        .join(format!("{CHILD}-001.md"))
}

#[test]
fn scan_verb_heals_a_crash_stranded_child() {
    let ws = workspace_with_crashed_child();
    // Hold the parent's executor lock across the scan: the sweep's
    // deposit is lock-free and still lands, while the flush observes a
    // driven branch and leaves it alone (§2.11) — so the deposit is
    // still in the inbox for this test to read. The flush's real
    // driver launch is exercised end-to-end in `advance_cli.rs`.
    let parent_inbox = ws.path().join("inbox").join(PARENT);
    let _held = lernie::prompt::inbox::try_acquire(&parent_inbox)
        .unwrap()
        .expect("free");
    let out = Command::new(lernie_bin())
        .arg("scan")
        .arg(ws.path())
        .output()
        .expect("spawn lernie scan");
    assert!(
        out.status.success(),
        "lernie scan: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // The operator summary on stdout.
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("silent deaths: 1"), "got {stdout:?}");
    // The healing: a died-epitaph result message deposited on the
    // crashed child's behalf, into its parent's inbox.
    let body = std::fs::read_to_string(died_deposit(ws.path())).unwrap();
    assert!(body.contains("epitaph: died"), "got {body:?}");
    assert!(body.contains(&format!("from: {CHILD}")), "got {body:?}");
}

#[test]
fn scan_verb_surfaces_a_broken_workspace_loudly() {
    // No `root` worktree → the branch enumeration fails → non-zero exit
    // (an operator verb is loud, §2.11).
    let ws = TempDir::new().unwrap();
    let out = Command::new(lernie_bin())
        .arg("scan")
        .arg(ws.path())
        .output()
        .expect("spawn lernie scan");
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("lernie scan"));
}

#[test]
fn prompt_hot_path_runs_no_workspace_scan() {
    // The same crashed-child workspace, touched by a driver instead of
    // the operator verb. `lernie prompt` fails fast here (LERNIE_HOME has
    // no models.yaml), but the point is what it must NOT have done first:
    // before bl-5846 the startup scan ran ahead of config load and would
    // have deposited the died epitaph; now no deposit may appear.
    let ws = workspace_with_crashed_child();
    let harness = TempDir::new().unwrap();
    let out = Command::new(lernie_bin())
        .arg("prompt")
        .arg(ws.path())
        .arg("hi")
        .env("LERNIE_HOME", harness.path())
        .output()
        .expect("spawn lernie prompt");
    assert!(!out.status.success(), "prompt fails on the empty harness");
    assert!(
        !died_deposit(ws.path()).exists(),
        "lernie prompt must not sweep the workspace (§2.11)"
    );
}

#[test]
fn dispatch_hot_path_runs_no_workspace_scan() {
    let ws = workspace_with_crashed_child();
    let out = Command::new(lernie_bin())
        .args(["dispatch", "no-such-role"])
        .arg(ws.path())
        .arg(PARENT)
        .output()
        .expect("spawn lernie dispatch");
    assert!(!out.status.success(), "unknown role is refused");
    assert!(
        !died_deposit(ws.path()).exists(),
        "lernie dispatch must not sweep the workspace (§2.11)"
    );
}
