//! Descriptions-always producer, end-to-end through `lernie new`
//! (ARCH §3.3). A populated data-root pool must produce a committed
//! `descriptions/**` tree on the new repo's initial `main`, so a
//! downstream branch's tools composer (§3.3, bl-9e96) can intersect a
//! role's declared tools against it.

use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Git env vars a hook-invoked test may inherit; scrub them so the
/// spawned `git` operates on the scaffolded tempdir, not the outer repo.
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

fn lernie_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_lernie"))
}

#[test]
fn descriptions_are_snapshotted_from_the_pool_and_committed() {
    // A populated pool under LERNIE_HOME (which collapses both roots):
    // one tool schema and one skill frontmatter.
    let home = TempDir::new().unwrap();
    let data = home.path();
    std::fs::create_dir_all(data.join("tools")).unwrap();
    std::fs::write(data.join("tools/bash.json"), r#"{"type":"object"}"#).unwrap();
    std::fs::create_dir_all(data.join("skills/bash")).unwrap();
    std::fs::write(
        data.join("skills/bash/SKILL.md"),
        "---\nname: bash\ndescription: Run a shell command.\n---\n# bash\n",
    )
    .unwrap();

    let holder = TempDir::new().unwrap();
    let dest = holder.path().join("conv");
    let mut cmd = Command::new(lernie_bin());
    scrub_git_env(&mut cmd);
    let out = cmd
        .arg("new")
        .arg(&dest)
        .env("LERNIE_HOME", data)
        .env("GIT_AUTHOR_NAME", "lernie-test")
        .env("GIT_AUTHOR_EMAIL", "test@example.invalid")
        .env("GIT_COMMITTER_NAME", "lernie-test")
        .env("GIT_COMMITTER_EMAIL", "test@example.invalid")
        .output()
        .expect("invoke lernie binary");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    // The tool schema is copied verbatim; the skill file carries only
    // the frontmatter body (fenced markdown stripped).
    let root = dest.join("root");
    assert_eq!(
        std::fs::read_to_string(root.join("descriptions/tools/bash.json")).unwrap(),
        r#"{"type":"object"}"#
    );
    assert_eq!(
        std::fs::read_to_string(root.join("descriptions/skills/bash.md")).unwrap(),
        "name: bash\ndescription: Run a shell command.\n"
    );

    // The snapshot is committed (tracked) on main, not worktree dirt.
    let mut ls = Command::new("git");
    scrub_git_env(&mut ls);
    let listed = String::from_utf8(
        ls.arg("-C")
            .arg(&root)
            .args(["ls-files"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    assert!(listed.lines().any(|l| l == "descriptions/tools/bash.json"));
    assert!(listed.lines().any(|l| l == "descriptions/skills/bash.md"));
}
