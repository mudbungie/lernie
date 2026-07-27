//! [`super::require_source`] — the `--from <source>` resolution that
//! precedes materialization (bl-55e0). Split from [`super::tests`] for
//! the per-file line cap.

use super::tests::{show, workspace, write_files};
use super::{Error, Origin, author, from_cli};
use crate::template::{GitRunner, RealGit};
use crate::workspace::repo_git;
use std::fs;
use tempfile::TempDir;

#[test]
fn forking_off_a_missing_lineage_names_the_lineages_that_exist() {
    // The decline is the product's, not git's — no plumbing argv, no
    // `.config-author`, no `config/` ref prefix — and it names the pool.
    // Resolution precedes materialization, so the transient checkout is
    // never created and the pass leaves no ref behind.
    let (holder, ws) = workspace();
    let no_pool = holder.path().join("no-pool");
    from_cli(
        &ws,
        &no_pool,
        Some("lone"),
        None,
        true,
        write_files(&[("goal-note.txt", "x\n")]),
        &RealGit::new(),
    )
    .unwrap();
    let err = from_cli(
        &ws,
        &no_pool,
        Some("x"),
        Some("nosuch"),
        false,
        write_files(&[("providers.yaml", "roles: {}\n")]),
        &RealGit::new(),
    )
    .unwrap_err();
    assert!(matches!(err, Error::NoSuchLineage(..)), "got {err:?}");
    assert_eq!(
        err.to_string(),
        "no config lineage \"nosuch\" in this workspace — existing lineages: default, lone"
    );
    assert!(!ws.join(".config-author").exists(), "checkout materialized");
    assert!(show(&ws, "config/x:providers.yaml").is_err(), "ref created");
}

#[test]
fn the_missing_lineage_decline_reads_none_when_no_lineage_exists() {
    // A `repo.git` with no `config/*` ref at all: the refusal still names
    // that there is nothing to fork from.
    let holder = TempDir::new().unwrap();
    let ws = holder.path().join("bare-ws");
    fs::create_dir_all(repo_git(&ws)).unwrap();
    RealGit::new()
        .run(&repo_git(&ws), &["init", "--bare", "--quiet", "."])
        .unwrap();
    let err = author(
        &ws,
        &holder.path().join("no-pool"),
        "x",
        Origin::Fork { source: "nosuch" },
        write_files(&[]),
        &RealGit::new(),
    )
    .unwrap_err();
    assert_eq!(
        err.to_string(),
        "no config lineage \"nosuch\" in this workspace — existing lineages: (none)"
    );
}

#[test]
fn an_unreadable_repo_surfaces_as_a_git_decline() {
    // `repo.git` is a directory but not a repository: the layout guard
    // passes and the lineage enumeration is what fails.
    let holder = TempDir::new().unwrap();
    let ws = holder.path().join("hollow-ws");
    fs::create_dir_all(repo_git(&ws)).unwrap();
    let err = author(
        &ws,
        &holder.path().join("no-pool"),
        "x",
        Origin::Fork { source: "default" },
        write_files(&[]),
        &RealGit::new(),
    )
    .unwrap_err();
    assert!(matches!(err, Error::Git(_)), "got {err:?}");
}
