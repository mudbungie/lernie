//! Config-commit authoring beyond `lernie new` (ARCH §2.2, §2.3).
//!
//! [`super::scaffold`] authors a workspace's *first* config commit — an
//! orphan root on `config/default`. This module is the general
//! harness-assisted user act §2.2 describes for *every* later config
//! commit: materialize a transient checkout of the target config
//! lineage, refresh the `descriptions/**` snapshot from the data-root
//! pools (§3.3 — the descriptions-always producer's ongoing home), hand
//! the checkout to an edit step, commit, and tear the checkout down.
//!
//! Three [`Origin`]s cover the §2.2/§2.3 cases: **advancing** an existing
//! config branch, **forking** a new one off an existing head, and
//! starting a fresh **orphan** lineage seeded from the embedded template.
//! Only this act moves a config branch (§2.3 branch-advancement
//! invariant); an agent's governing config is derived from ancestry, so
//! a new commit here governs only agents forked after it ("fork is the
//! freeze", §2.2).
//!
//! The [`author`] core is non-interactive: the `edit` closure is the
//! seam the `lernie config` bin fills with a `$EDITOR` hand-off and tests
//! fill with direct writes, so the machinery stays fully covered while
//! the untestable interactive sliver lives at the bin (ARCH §3.4).

use super::{GitRunner, TEMPLATE, descriptions};
use crate::workspace::{self, config_ref};
use std::io;
use std::path::Path;

/// Transient authoring checkout under the workspace root; removed (as a
/// git worktree) once the commit lands. One at a time — authoring a
/// config commit is a deliberate single-user act (ARCH §1.1).
const AUTHOR_DIR: &str = ".config-author";

/// Which config lineage an authoring pass targets (ARCH §2.2, §2.3).
pub enum Origin<'a> {
    /// Advance the existing `config/<name>` branch: the checkout starts
    /// at its head and the commit lands back on it.
    Advance,
    /// Fork a new `config/<name>` off the head of `config/<source>`
    /// (§2.2 "further config branches fork from existing ones").
    Fork { source: &'a str },
    /// Start `config/<name>` as a fresh orphan lineage, seeded from the
    /// embedded control-file template (§2.2 "or start fresh").
    Orphan,
}

/// Why [`author`] could not complete.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The workspace-layout guard ([`workspace::require`]) declined the
    /// path — not a workspace, or the retired per-conversation layout.
    #[error(transparent)]
    Layout(#[from] workspace::LayoutError),
    /// Filesystem error preparing the checkout (template extraction, the
    /// checkout directory).
    #[error("config authoring I/O: {0}")]
    Io(#[source] io::Error),
    /// A `git` step failed — including the loud declines git itself
    /// raises for the illegal cases (advancing a branch that does not
    /// exist, forking onto a name that already does, an empty commit).
    #[error("config authoring git: {0}")]
    Git(#[source] io::Error),
    /// The `descriptions/**` refresh from the data-root pools failed
    /// (ARCH §3.3).
    #[error("descriptions-always: {0}")]
    Descriptions(#[source] descriptions::Error),
    /// The edit step (the `$EDITOR` hand-off, or a test's writer) failed.
    #[error("edit step: {0}")]
    Edit(#[source] io::Error),
    /// `--from` and `--orphan` were both given — a new branch is either a
    /// fork of a source or a fresh lineage, never both.
    #[error("pass --from <source> or --orphan, not both")]
    Conflict,
}

/// Resolve the [`Origin`] from `lernie config`'s flags and run [`author`]
/// — the testable body of the verb (ARCH §3.4). `--from` forks,
/// `--orphan` starts fresh, neither advances; both together is
/// [`Error::Conflict`]. `name` defaults to `default`. The bin supplies
/// the resolved `data_root` and the `$EDITOR` `edit` hand-off; nothing
/// here is interactive.
pub fn from_cli<G: GitRunner>(
    workspace: &Path,
    data_root: &Path,
    name: Option<&str>,
    from: Option<&str>,
    orphan: bool,
    edit: impl FnOnce(&Path) -> io::Result<()>,
    git: &G,
) -> Result<(), Error> {
    let origin = match (from, orphan) {
        (Some(source), false) => Origin::Fork { source },
        (None, true) => Origin::Orphan,
        (None, false) => Origin::Advance,
        (Some(_), true) => return Err(Error::Conflict),
    };
    let name = name.unwrap_or("default");
    author(workspace, data_root, name, origin, edit, git)
}

/// Author one config commit onto `config/<name>` (ARCH §2.2). Guards the
/// workspace layout, materializes the checkout per `origin`, refreshes
/// the `descriptions/**` snapshot from the `data_root` pools (§3.3), runs
/// `edit` against the checkout, commits, and tears the checkout down.
///
/// `edit` receives the checkout path; whatever it writes becomes the new
/// config commit's content on top of the origin's tree. A no-op edit that
/// leaves the tree unchanged is declined by git's empty-commit refusal
/// (surfaced as [`Error::Git`]) — an authoring pass that changes nothing
/// does not move the branch.
pub fn author<G: GitRunner>(
    workspace: &Path,
    data_root: &Path,
    name: &str,
    origin: Origin,
    edit: impl FnOnce(&Path) -> io::Result<()>,
    git: &G,
) -> Result<(), Error> {
    workspace::require(workspace)?;
    let repo = workspace::repo_git(workspace);
    let author = workspace.join(AUTHOR_DIR);
    let author_str = author.to_string_lossy().to_string();

    materialize(git, &repo, &config_ref(name), &author_str, &origin)?;
    // `git worktree add` makes the dir in production; explicit for the
    // stub-git tests (a harmless no-op otherwise) — as in `scaffold`.
    std::fs::create_dir_all(&author).map_err(Error::Io)?;
    if matches!(origin, Origin::Orphan) {
        TEMPLATE.extract(&author).map_err(Error::Io)?;
    }
    descriptions::snapshot(data_root, &author).map_err(Error::Descriptions)?;
    edit(&author).map_err(Error::Edit)?;
    super::commit_checkout(git, &repo, &author, &commit_message(name, &origin))
        .map_err(Error::Git)?;
    Ok(())
}

/// Create the authoring checkout at `author_str` for the given origin:
/// check out the existing branch (advance), branch off a source head
/// (fork), or open a fresh orphan branch (orphan). Wrong-existence cases
/// are git's to decline (invalid reference / branch already exists).
fn materialize<G: GitRunner>(
    git: &G,
    repo: &Path,
    target: &str,
    author_str: &str,
    origin: &Origin,
) -> Result<(), Error> {
    // `src` outlives the args it feeds; empty unless this is a fork.
    let src = match origin {
        Origin::Fork { source } => config_ref(source),
        _ => String::new(),
    };
    let args: Vec<&str> = match origin {
        Origin::Advance => vec!["worktree", "add", author_str, target],
        Origin::Fork { .. } => vec!["worktree", "add", "-b", target, author_str, &src],
        Origin::Orphan => vec!["worktree", "add", "--orphan", "-b", target, author_str],
    };
    git.run(repo, &args).map_err(Error::Git)
}

/// The commit subject, naming the act and the branch it lands on — the
/// same `config: …` convention `scaffold` uses for the first commit.
fn commit_message(name: &str, origin: &Origin) -> String {
    let target = config_ref(name);
    match origin {
        Origin::Advance => format!("config: advance [{target}]"),
        Origin::Fork { source } => format!("config: fork {} [{target}]", config_ref(source)),
        Origin::Orphan => format!("config: init [{target}]"),
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_stub;
