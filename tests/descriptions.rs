//! Descriptions-always producer, end-to-end through `lernie new`
//! (ARCH §3.3). A populated data-root pool must produce a committed
//! `descriptions/**` tree in the workspace's first config commit
//! (`config/default`, §2.2), so a downstream branch's tools composer
//! (§3.3, bl-9e96) can intersect a role's declared tools against it.

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

    // The snapshot is committed in the config commit's tree (§2.2):
    // the tool schema verbatim, the skill file carrying only the
    // frontmatter body (fenced markdown stripped).
    let repo = dest.join("repo.git");
    let show = |path: &str| -> String {
        let mut cmd = Command::new("git");
        scrub_git_env(&mut cmd);
        let out = cmd
            .arg("-C")
            .arg(&repo)
            .args(["show", &format!("config/default:{path}")])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git show {path}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    };
    assert_eq!(show("descriptions/tools/bash.json"), r#"{"type":"object"}"#);
    assert_eq!(
        show("descriptions/skills/bash.md"),
        "name: bash\ndescription: Run a shell command.\n"
    );
}
