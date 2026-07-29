//! Unit tests for the dispatch-time descriptor prune (ARCH §3.3, §5.1).
//!
//! Lives in a sibling file rather than an inline `mod tests` so the
//! production module stays under the 300-line repo cap.

use super::*;
use std::cell::RefCell;
use std::io;
use std::path::PathBuf;

#[derive(Default)]
struct StubGit {
    runs: RefCell<Vec<(PathBuf, Vec<String>)>>,
    fail: bool,
}

impl GitRunner for StubGit {
    fn run(&self, dest: &Path, args: &[&str]) -> io::Result<()> {
        self.runs.borrow_mut().push((
            dest.to_path_buf(),
            args.iter().map(|s| (*s).to_owned()).collect(),
        ));
        if self.fail {
            Err(io::Error::other("stub git fail"))
        } else {
            Ok(())
        }
    }
    fn run_capture(&self, _dest: &Path, _args: &[&str]) -> io::Result<String> {
        unreachable!("the prune never issues capturing git ops")
    }
}

/// A worktree carrying the shipped five-tool snapshot plus one standalone
/// skill (no tool claims it) — the shape a fresh root inherits from the
/// config commit (§3.3 *Descriptions-always population*).
fn snapshotted() -> tempfile::TempDir {
    let dir = tempfile::TempDir::new().unwrap();
    let tools = dir.path().join(TOOLS_DIR);
    let skills = dir.path().join(SKILLS_DIR);
    std::fs::create_dir_all(&tools).unwrap();
    std::fs::create_dir_all(&skills).unwrap();
    for name in ["bash", "dispatch", "load_skill", "message", "read_file"] {
        std::fs::write(tools.join(format!("{name}.json")), "{}").unwrap();
        std::fs::write(skills.join(format!("{name}.md")), "name: x").unwrap();
    }
    std::fs::write(skills.join("housekeeping.md"), "name: housekeeping").unwrap();
    dir
}

fn granted(names: &[&str]) -> Vec<String> {
    names.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn a_grant_covering_the_whole_snapshot_issues_no_git_op() {
    let wt = snapshotted();
    let git = StubGit::default();
    prune_ungranted(
        wt.path(),
        &granted(&["bash", "dispatch", "load_skill", "message", "read_file"]),
        &git,
    )
    .unwrap();
    assert!(
        git.runs.borrow().is_empty(),
        "the shipped default prunes nothing and must cost nothing"
    );
}

#[test]
fn ungranted_tools_lose_schema_and_claimed_skill_together() {
    let wt = snapshotted();
    let git = StubGit::default();
    prune_ungranted(wt.path(), &granted(&["bash", "read_file"]), &git).unwrap();

    let runs = git.runs.borrow();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].0, wt.path());
    assert_eq!(
        runs[0].1,
        vec![
            "rm",
            "-q",
            "--ignore-unmatch",
            "--",
            "descriptions/tools/dispatch.json",
            "descriptions/skills/dispatch.md",
            "descriptions/tools/load_skill.json",
            "descriptions/skills/load_skill.md",
            "descriptions/tools/message.json",
            "descriptions/skills/message.md",
        ],
        "sorted, schema-then-skill, and nothing granted is named"
    );
    // The standalone skill no tool claims is never named: it composes as
    // a head text block and is `load_skill`-able (§3.3 two wire homes).
    assert!(!runs[0].1.iter().any(|a| a.contains("housekeeping")));
}

#[test]
fn an_empty_grant_prunes_the_whole_snapshot() {
    // The compactor's shape (§2.7): `tools:` empty, its own pair injected
    // by the procedure and never riding `descriptions/**`.
    let wt = snapshotted();
    let git = StubGit::default();
    prune_ungranted(wt.path(), &[], &git).unwrap();
    let runs = git.runs.borrow();
    assert_eq!(runs[0].1.len(), 4 + 5 * 2);
}

#[test]
fn a_tree_with_no_snapshot_is_a_no_op() {
    let wt = tempfile::TempDir::new().unwrap();
    let git = StubGit::default();
    prune_ungranted(wt.path(), &[], &git).unwrap();
    assert!(git.runs.borrow().is_empty());
}

#[test]
fn non_schema_entries_in_the_tools_dir_are_ignored() {
    let wt = tempfile::TempDir::new().unwrap();
    let tools = wt.path().join(TOOLS_DIR);
    std::fs::create_dir_all(&tools).unwrap();
    std::fs::write(tools.join("README"), "not a schema").unwrap();
    let git = StubGit::default();
    prune_ungranted(wt.path(), &[], &git).unwrap();
    assert!(git.runs.borrow().is_empty());
}

#[test]
fn an_unreadable_snapshot_dir_surfaces_rather_than_pruning_blind() {
    // A regular file where `descriptions/tools/` must be: `read_dir`
    // fails with something other than NotFound, and a prune that cannot
    // enumerate must not silently decide nothing is stranded.
    let wt = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(wt.path().join("descriptions")).unwrap();
    std::fs::write(wt.path().join(TOOLS_DIR), b"not a directory").unwrap();
    let git = StubGit::default();
    let err = prune_ungranted(wt.path(), &[], &git).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn a_failing_git_rm_surfaces_as_a_named_git_error() {
    let wt = snapshotted();
    let git = StubGit {
        fail: true,
        ..StubGit::default()
    };
    let err = prune_ungranted(wt.path(), &[], &git).unwrap_err();
    assert!(
        matches!(&err, Error::Git { op, .. } if *op == "rm ungranted descriptors"),
        "got {err:?}"
    );
}
