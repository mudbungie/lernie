//! [`super::author`] tests: the three origins end-to-end with real git,
//! and the failure arms that need no stub — the layout guard, git's own
//! declines, the descriptions refresh, and the edit step. The teardown
//! and declined-pass contract is [`super::tests_teardown`]; the stubbed
//! Io arms are [`super::tests_stub`]; the `--from` source resolution is
//! [`super::tests_lineage`].

use super::{Error, Origin, author, from_cli};
use crate::template::{GitRunner, RealGit, scaffold};
use crate::workspace::{config_ref, repo_git};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// A scaffolded workspace with an empty pool. Returns `(holder, ws)`.
pub(super) fn workspace() -> (TempDir, PathBuf) {
    let holder = TempDir::new().unwrap();
    let ws = holder.path().join("ws");
    let roots = crate::test_support::bare_roots(holder.path());
    scaffold(&ws, &roots, &RealGit::new()).unwrap();
    (holder, ws)
}

/// An `edit` closure that writes each `(rel, content)` file.
pub(super) fn write_files(files: &[(&str, &str)]) -> impl FnOnce(&Path) -> io::Result<()> {
    let owned: Vec<(String, String)> = files
        .iter()
        .map(|(r, c)| (r.to_string(), c.to_string()))
        .collect();
    move |dir: &Path| {
        for (rel, content) in &owned {
            let path = dir.join(rel);
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(path, content)?;
        }
        Ok(())
    }
}

pub(super) fn show(ws: &Path, spec: &str) -> io::Result<String> {
    RealGit::new().run_capture(&repo_git(ws), &["show", spec])
}

#[test]
fn advance_lands_a_new_commit_on_the_existing_branch() {
    let (holder, ws) = workspace();
    let before = show(&ws, "config/default:version").unwrap();
    author(
        &ws,
        &holder.path().join("no-pool"),
        "default",
        Origin::Advance,
        write_files(&[("providers.yaml", "roles: {}\n")]),
        &RealGit::new(),
    )
    .unwrap();
    // The branch advanced and carries the edit; the checkout is gone.
    assert_eq!(
        show(&ws, "config/default:providers.yaml").unwrap(),
        "roles: {}"
    );
    assert_eq!(show(&ws, "config/default:version").unwrap(), before);
    assert!(!ws.join(".config-author").exists());
}

#[test]
fn fork_creates_a_new_branch_off_the_source_head() {
    let (holder, ws) = workspace();
    author(
        &ws,
        &holder.path().join("no-pool"),
        "strict-verifier",
        Origin::Fork { source: "default" },
        write_files(&[("providers.yaml", "roles: {strict: true}\n")]),
        &RealGit::new(),
    )
    .unwrap();
    // The new branch exists, shares default's ancestry (not an orphan),
    // and carries the fork's edit.
    assert_eq!(
        show(&ws, "config/strict-verifier:providers.yaml").unwrap(),
        "roles: {strict: true}"
    );
    RealGit::new()
        .run(
            &repo_git(&ws),
            &["merge-base", "config/default", "config/strict-verifier"],
        )
        .expect("fork shares ancestry with its source");
}

#[test]
fn orphan_starts_a_fresh_lineage_from_the_template() {
    let (holder, ws) = workspace();
    author(
        &ws,
        &holder.path().join("no-pool"),
        "scratch",
        Origin::Orphan,
        write_files(&[("goal-note.txt", "fresh\n")]),
        &RealGit::new(),
    )
    .unwrap();
    // The orphan carries the embedded template control files plus the
    // edit, and shares NO ancestor with config/default (§2.2).
    assert!(
        !show(&ws, "config/scratch:workflow.yaml")
            .unwrap()
            .is_empty()
    );
    assert_eq!(show(&ws, "config/scratch:goal-note.txt").unwrap(), "fresh");
    let merge_base = RealGit::new().run(
        &repo_git(&ws),
        &["merge-base", "config/default", "config/scratch"],
    );
    assert!(merge_base.is_err(), "orphan must share no ancestry");
}

#[test]
fn authoring_refreshes_the_descriptions_snapshot_from_the_pools() {
    let (holder, ws) = workspace();
    // A tool schema appears in the data-root pool after creation; the
    // authoring pass snapshots it into descriptions/** (§3.3).
    let data_root = holder.path().join("data");
    fs::create_dir_all(data_root.join("tools")).unwrap();
    fs::write(data_root.join("tools/echo.json"), "{\"k\":1}").unwrap();
    author(
        &ws,
        &data_root,
        "default",
        Origin::Advance,
        write_files(&[("providers.yaml", "roles: {}\n")]),
        &RealGit::new(),
    )
    .unwrap();
    assert_eq!(
        show(&ws, "config/default:descriptions/tools/echo.json").unwrap(),
        "{\"k\":1}"
    );
}

#[test]
fn layout_guard_declines_a_non_workspace() {
    let holder = TempDir::new().unwrap();
    let err = author(
        holder.path(),
        &holder.path().join("no-pool"),
        "default",
        Origin::Advance,
        write_files(&[]),
        &RealGit::new(),
    )
    .unwrap_err();
    assert!(matches!(err, Error::Layout(_)), "got {err:?}");
}

#[test]
fn advancing_a_missing_branch_is_a_git_decline() {
    let (holder, ws) = workspace();
    let err = author(
        &ws,
        &holder.path().join("no-pool"),
        "ghost",
        Origin::Advance,
        write_files(&[("providers.yaml", "x\n")]),
        &RealGit::new(),
    )
    .unwrap_err();
    assert!(matches!(err, Error::Git(_)), "got {err:?}");
}

#[test]
fn descriptions_producer_failure_surfaces() {
    let (holder, ws) = workspace();
    let data_root = holder.path().join("data");
    fs::create_dir_all(data_root.join("skills/broken")).unwrap();
    fs::write(data_root.join("skills/broken/SKILL.md"), "no frontmatter\n").unwrap();
    let err = author(
        &ws,
        &data_root,
        "default",
        Origin::Advance,
        write_files(&[]),
        &RealGit::new(),
    )
    .unwrap_err();
    assert!(matches!(err, Error::Descriptions(_)), "got {err:?}");
}

#[test]
fn edit_step_failure_surfaces() {
    let (holder, ws) = workspace();
    let err = author(
        &ws,
        &holder.path().join("no-pool"),
        "default",
        Origin::Advance,
        |_dir: &Path| Err(io::Error::other("editor blew up")),
        &RealGit::new(),
    )
    .unwrap_err();
    assert!(matches!(err, Error::Edit(_)), "got {err:?}");
}

// --- from_cli: flag → origin resolution ----------------------------

#[test]
fn from_cli_defaults_name_and_advances() {
    let (holder, ws) = workspace();
    from_cli(
        &ws,
        &holder.path().join("no-pool"),
        None,
        None,
        false,
        write_files(&[("providers.yaml", "roles: {}\n")]),
        &RealGit::new(),
    )
    .unwrap();
    assert_eq!(
        show(&ws, "config/default:providers.yaml").unwrap(),
        "roles: {}"
    );
}

#[test]
fn from_cli_forks_with_from_and_orphans_with_orphan() {
    let (holder, ws) = workspace();
    let no_pool = holder.path().join("no-pool");
    from_cli(
        &ws,
        &no_pool,
        Some("forked"),
        Some("default"),
        false,
        write_files(&[("providers.yaml", "roles: {f: 1}\n")]),
        &RealGit::new(),
    )
    .unwrap();
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
    assert!(show(&ws, "config/forked:providers.yaml").is_ok());
    assert!(show(&ws, "config/lone:goal-note.txt").is_ok());
}

#[test]
fn from_cli_declines_from_and_orphan_together() {
    let (holder, ws) = workspace();
    let err = from_cli(
        &ws,
        &holder.path().join("no-pool"),
        Some("x"),
        Some("default"),
        true,
        write_files(&[]),
        &RealGit::new(),
    )
    .unwrap_err();
    assert!(matches!(err, Error::Conflict), "got {err:?}");
}

#[test]
fn commit_message_names_the_act_and_branch() {
    assert_eq!(
        super::commit_message("default", &Origin::Advance),
        "config: advance [config/default]"
    );
    assert_eq!(
        super::commit_message("strict", &Origin::Fork { source: "default" }),
        format!("config: fork {} [config/strict]", config_ref("default"))
    );
    assert_eq!(
        super::commit_message("scratch", &Origin::Orphan),
        "config: init [config/scratch]"
    );
}
