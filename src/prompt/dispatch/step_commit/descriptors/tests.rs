//! Unit tests for the dispatch-time descriptor derivation (ARCH §3.3,
//! §5.1).
//!
//! Lives in a sibling file rather than an inline `mod tests` so the
//! production module stays under the 300-line repo cap.

use super::*;
use std::cell::RefCell;
use std::io;
use std::path::PathBuf;

/// Recording [`GitRunner`]. `absent` names `<commit>:<path>` specs the
/// config commit does *not* carry (a failing `cat-file -e`), `fail_op`
/// the first argument whose command fails, and `listing` the `ls-tree`
/// reply the decline path reads (`None` = the listing is unreadable).
struct StubGit {
    runs: RefCell<Vec<(PathBuf, Vec<String>)>>,
    absent: Vec<String>,
    fail_op: Option<&'static str>,
    listing: Option<String>,
}

impl Default for StubGit {
    fn default() -> Self {
        Self {
            runs: RefCell::new(Vec::new()),
            absent: Vec::new(),
            fail_op: None,
            listing: Some(String::new()),
        }
    }
}

impl StubGit {
    fn absent(paths: &[&str]) -> Self {
        Self {
            absent: paths.iter().map(|p| (*p).to_owned()).collect(),
            ..Self::default()
        }
    }
    fn failing(op: &'static str) -> Self {
        Self {
            fail_op: Some(op),
            ..Self::default()
        }
    }
    /// Every recorded command whose first argument is `op`.
    fn ops(&self, op: &str) -> Vec<Vec<String>> {
        self.runs
            .borrow()
            .iter()
            .filter(|(_, args)| args.first().is_some_and(|a| a == op))
            .map(|(_, args)| args.clone())
            .collect()
    }
}

impl GitRunner for StubGit {
    fn run(&self, dest: &Path, args: &[&str]) -> io::Result<()> {
        self.runs.borrow_mut().push((
            dest.to_path_buf(),
            args.iter().map(|s| (*s).to_owned()).collect(),
        ));
        if args.first() == Some(&"cat-file") {
            let spec = args.last().copied().unwrap_or_default();
            let missing = self.absent.iter().any(|a| spec.ends_with(a.as_str()));
            return if missing {
                Err(io::Error::other("not in tree"))
            } else {
                Ok(())
            };
        }
        if self.fail_op == args.first().copied() {
            return Err(io::Error::other("stub git fail"));
        }
        Ok(())
    }
    fn run_capture(&self, _dest: &Path, _args: &[&str]) -> io::Result<String> {
        self.listing
            .clone()
            .ok_or_else(|| io::Error::other("stub ls-tree fail"))
    }
}

/// The config-commit sha stand-in — what a real derivation names.
const COMMIT: &str = "c0ffee";

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

fn tools(names: &[&str]) -> Vec<String> {
    names.iter().map(|s| (*s).to_owned()).collect()
}

fn grant<'a>(role: &'a str, tools: &'a [String]) -> Grant<'a> {
    Grant {
        role,
        tools,
        config_commit: COMMIT,
    }
}

#[test]
fn a_grant_covering_the_whole_snapshot_drops_nothing() {
    let wt = snapshotted();
    let git = StubGit::default();
    let t = tools(&["bash", "dispatch", "load_skill", "message", "read_file"]);
    derive(wt.path(), &grant("worker", &t), &git).unwrap();
    assert!(
        git.ops("rm").is_empty(),
        "the shipped default strands nothing, so nothing is removed"
    );
    // Every granted tool is still checked out from the config commit:
    // the tree it forked into is never the authority (§3.3).
    assert_eq!(git.ops("checkout")[0].len(), 3 + 5 * 2);
}

#[test]
fn ungranted_tools_lose_schema_and_claimed_skill_together() {
    let wt = snapshotted();
    let git = StubGit::default();
    let t = tools(&["bash", "read_file"]);
    derive(wt.path(), &grant("worker", &t), &git).unwrap();

    assert_eq!(
        git.ops("rm")[0],
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
    assert!(!git.ops("rm")[0].iter().any(|a| a.contains("housekeeping")));
}

#[test]
fn a_grant_the_forked_tree_lacks_is_checked_out_from_the_config_commit() {
    // bl-a900: a child's grant is not capped by what its dispatcher's
    // tree carried. Here the fork point carried no descriptors at all —
    // a parent whose own grant was narrow — and the child still gets its
    // full toolset, derived from the governing config commit.
    let wt = tempfile::TempDir::new().unwrap();
    let git = StubGit::default();
    let t = tools(&["message"]);
    derive(wt.path(), &grant("sensor", &t), &git).unwrap();
    assert!(git.ops("rm").is_empty());
    assert_eq!(
        git.ops("checkout")[0],
        vec![
            "checkout",
            COMMIT,
            "--",
            "descriptions/tools/message.json",
            "descriptions/skills/message.md",
        ]
    );
}

#[test]
fn a_tool_with_no_snapshotted_frontmatter_checks_out_its_schema_alone() {
    // §3.3 sanctions the schema-without-frontmatter state (the entry
    // composes with no top-level `description`), so it is not a decline.
    let wt = tempfile::TempDir::new().unwrap();
    let git = StubGit::absent(&["descriptions/skills/message.md"]);
    let t = tools(&["message"]);
    derive(wt.path(), &grant("sensor", &t), &git).unwrap();
    assert_eq!(
        git.ops("checkout")[0],
        vec!["checkout", COMMIT, "--", "descriptions/tools/message.json"]
    );
}

#[test]
fn an_empty_grant_drops_the_whole_snapshot_and_checks_out_nothing() {
    // The compactor's shape (§2.7): `tools:` empty, its own pair injected
    // by the procedure and never riding `descriptions/**`.
    let wt = snapshotted();
    let git = StubGit::default();
    derive(wt.path(), &grant("compactor", &[]), &git).unwrap();
    assert_eq!(git.ops("rm")[0].len(), 4 + 5 * 2);
    assert!(git.ops("checkout").is_empty());
}

#[test]
fn an_undescribed_grant_declines_naming_the_tool_and_the_pool() {
    // The other half of bl-a900: a granted tool with no descriptor
    // anywhere in the governing config commit is refused, not composed
    // silently smaller — and the refusal changes nothing on disk.
    let wt = snapshotted();
    let mut git = StubGit::absent(&["descriptions/tools/slack_read.json"]);
    git.listing = Some("bash.json\nmessage.json\nREADME\n".to_string());
    let t = tools(&["message", "slack_read"]);
    let err = derive(wt.path(), &grant("sensor", &t), &git).unwrap_err();
    match &err {
        Error::GrantUndescribed(u) => {
            assert_eq!((u.role.as_str(), u.tool.as_str()), ("sensor", "slack_read"));
            assert_eq!(u.described, "bash, message");
        }
        other => panic!("got {other:?}"),
    }
    assert!(err.to_string().contains("slack_read"), "{err}");
    assert!(git.ops("rm").is_empty() && git.ops("checkout").is_empty());
}

#[test]
fn an_unreadable_pool_listing_still_declines_naming_the_empty_pool() {
    let wt = tempfile::TempDir::new().unwrap();
    let mut git = StubGit::absent(&["descriptions/tools/slack_read.json"]);
    git.listing = None;
    let t = tools(&["slack_read"]);
    let err = derive(wt.path(), &grant("sensor", &t), &git).unwrap_err();
    assert!(matches!(&err, Error::GrantUndescribed(u) if u.described == "(none)"));
}

#[test]
fn a_tree_with_no_snapshot_and_no_grant_is_a_no_op() {
    let wt = tempfile::TempDir::new().unwrap();
    let git = StubGit::default();
    derive(wt.path(), &grant("compactor", &[]), &git).unwrap();
    assert!(git.runs.borrow().is_empty());
}

#[test]
fn non_schema_entries_in_the_tools_dir_are_ignored() {
    let wt = tempfile::TempDir::new().unwrap();
    let tools_dir = wt.path().join(TOOLS_DIR);
    std::fs::create_dir_all(&tools_dir).unwrap();
    std::fs::write(tools_dir.join("README"), "not a schema").unwrap();
    let git = StubGit::default();
    derive(wt.path(), &grant("compactor", &[]), &git).unwrap();
    assert!(git.runs.borrow().is_empty());
}

#[test]
fn an_unreadable_snapshot_dir_surfaces_rather_than_dropping_blind() {
    // A regular file where `descriptions/tools/` must be: `read_dir`
    // fails with something other than NotFound, and a derivation that
    // cannot enumerate must not silently decide nothing is stranded.
    let wt = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(wt.path().join("descriptions")).unwrap();
    std::fs::write(wt.path().join(TOOLS_DIR), b"not a directory").unwrap();
    let git = StubGit::default();
    let err = derive(wt.path(), &grant("compactor", &[]), &git).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}

#[test]
fn a_failing_git_rm_surfaces_as_a_named_git_error() {
    let wt = snapshotted();
    let git = StubGit::failing("rm");
    let err = derive(wt.path(), &grant("compactor", &[]), &git).unwrap_err();
    assert!(
        matches!(&err, Error::Git { op, .. } if *op == "rm ungranted descriptors"),
        "got {err:?}"
    );
}

#[test]
fn a_failing_checkout_surfaces_as_a_named_git_error() {
    let wt = tempfile::TempDir::new().unwrap();
    let git = StubGit::failing("checkout");
    let t = tools(&["message"]);
    let err = derive(wt.path(), &grant("sensor", &t), &git).unwrap_err();
    assert!(
        matches!(&err, Error::Git { op, .. } if *op == "checkout granted descriptors"),
        "got {err:?}"
    );
}
