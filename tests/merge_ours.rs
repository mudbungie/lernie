//! ARCH §2.6 alignment check, end-to-end against real git.
//!
//! Phase 5 of the v0.3 layout migration calls this out as the
//! load-bearing test "without [which] merge=ours can silently
//! regress" — and indeed the previous implementation *had* silently
//! regressed (the `merge.ours.driver` config was unregistered and
//! the only-theirs-add case was unhandled).
//!
//! The fixture mirrors the dispatch shape of a real subagent merge:
//!   - scaffold a conv repo via `lernie new`, with `.gitattributes`
//!     + driver registered on `main` inside `root/`,
//!   - manually create a sibling subagent worktree on a `<root>-<sub>`
//!     branch off `main`,
//!   - in the subagent worktree, write `goal.md`, `soul.md`,
//!     `summary/001.md`, and `steps/<sub-id>/001/request.json`, then
//!     commit (the dispatch+response shape),
//!   - in `root/` (main), write a different `goal.md` and commit
//!     so the merge faces an add/add case as well as only-theirs-add
//!     cases,
//!   - call [`rebase_and_merge`] (the production merge protocol) to
//!     fold the subagent into main.
//!
//! The post-merge tree on main must satisfy ARCH §2.6:
//!   1. `goal.md` carries main's text — the subagent's version is
//!      gone, both because of the explicit alignment commit and as
//!      a backstop because of `merge=ours` in `.gitattributes`.
//!   2. `summary/001.md` is **not** present — `summary/**` is pinned
//!      to the parent's pre-merge state, which had nothing under it.
//!   3. `soul.md` is **not** present for the same reason.
//!   4. `steps/<sub-id>/001/request.json` **is** present —
//!      `steps/<sub-id>/` is explicitly *not* merge=ours per ARCH
//!      §2.6, and step records cross up into the parent.

use lernie::prompt::merge::rebase_and_merge;
use lernie::template::RealGit;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

fn lernie_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_lernie"))
}

/// Git env vars that a hook-invoked test may inherit. They would
/// cause subcommands to operate on the outer repo instead of the
/// scaffolded tempdir, so scrub them from every `git` we spawn here.
const INHERITED_GIT_ENV: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_PREFIX",
    "GIT_COMMON_DIR",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
];

fn scrub_git_env(cmd: &mut Command) {
    for var in INHERITED_GIT_ENV {
        cmd.env_remove(var);
    }
}

fn scaffold(dest: &Path) {
    let mut cmd = Command::new(lernie_bin());
    scrub_git_env(&mut cmd);
    let out = cmd
        .arg("new")
        .arg(dest)
        .env("GIT_AUTHOR_NAME", "lernie-test")
        .env("GIT_AUTHOR_EMAIL", "test@example.invalid")
        .env("GIT_COMMITTER_NAME", "lernie-test")
        .env("GIT_COMMITTER_EMAIL", "test@example.invalid")
        .output()
        .expect("invoke lernie binary");
    assert!(
        out.status.success(),
        "lernie new failed: {}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
}

fn run_git(dir: &Path, args: &[&str]) {
    let mut cmd = Command::new("git");
    scrub_git_env(&mut cmd);
    let out = cmd
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_AUTHOR_NAME", "lernie-test")
        .env("GIT_AUTHOR_EMAIL", "test@example.invalid")
        .env("GIT_COMMITTER_NAME", "lernie-test")
        .env("GIT_COMMITTER_EMAIL", "test@example.invalid")
        .output()
        .expect("spawn git");
    assert!(
        out.status.success(),
        "git {args:?} in {dir:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn merge_ours_discipline_holds_against_subagent_merge_back() {
    let holder = TempDir::new().unwrap();
    let repo = holder.path().join("conv");
    scaffold(&repo);
    let primary = repo.join("root");

    // Sibling subagent worktree at `<repo>/<sub-branch>/` (ARCH §2.2):
    // hyphenated descent, branch and dir share the name.
    let sub_branch = "main-sub";
    let sub_wt = repo.join(sub_branch);
    run_git(
        &primary,
        &[
            "worktree",
            "add",
            "-b",
            sub_branch,
            sub_wt.to_str().unwrap(),
            "main",
        ],
    );

    // Subagent dispatch shape: goal/soul/summary at worktree root,
    // step records under steps/<sub-id>/.
    std::fs::write(sub_wt.join("goal.md"), "subagent goal").unwrap();
    std::fs::write(sub_wt.join("soul.md"), "subagent soul").unwrap();
    std::fs::create_dir_all(sub_wt.join("summary")).unwrap();
    std::fs::write(sub_wt.join("summary/001.md"), "subagent summary 1").unwrap();
    std::fs::create_dir_all(sub_wt.join("steps/sub-id/001")).unwrap();
    std::fs::write(
        sub_wt.join("steps/sub-id/001/request.json"),
        b"{\"sub\": true}",
    )
    .unwrap();
    run_git(&sub_wt, &["add", "-A"]);
    run_git(&sub_wt, &["commit", "-m", "subagent dispatch+response"]);

    // Main writes its own goal.md so the merge faces an add/add
    // case (without an alignment step or driver, that resolves with
    // a hard conflict; the discipline must keep main's version).
    std::fs::write(primary.join("goal.md"), "main goal").unwrap();
    run_git(&primary, &["add", "goal.md"]);
    run_git(&primary, &["commit", "-m", "main goal"]);

    // Production merge protocol — rebase + alignment + --no-ff
    // merge + worktree remove. `repo` arg is the cwd for
    // `worktree remove`; since the conv-repo root itself is not a
    // git checkout (the `.git` lives inside `root/`, ARCH §2.2),
    // pass the primary worktree, which shares the same `.git` dir.
    rebase_and_merge(
        &primary,
        "main",
        &primary,
        &sub_wt,
        sub_branch,
        &RealGit::new(),
    )
    .expect("rebase_and_merge");

    // Post-merge state on main:
    let goal = std::fs::read_to_string(primary.join("goal.md")).unwrap();
    assert_eq!(goal, "main goal", "merge=ours kept main's goal.md");
    assert!(
        !primary.join("summary").exists() && !primary.join("summary/001.md").exists(),
        "subagent's summary/** must not propagate to main"
    );
    assert!(
        !primary.join("soul.md").exists(),
        "subagent's soul.md must not propagate to main"
    );
    assert!(
        primary.join("steps/sub-id/001/request.json").is_file(),
        "subagent's steps/<sub-id>/ tree must cross up into main"
    );
}
