//! The teardown contract (ARCH §2.2): a pass tears its checkout down
//! however it ends, a declined pass is a clean outcome that leaves the
//! workspace untouched and the verb immediately re-runnable, a failed
//! `--from` leaves no ref, and a killed pass's debris is healed by the
//! next pass rather than wedging it (§2.11).

use super::tests::{show, workspace, write_files};
use super::{Error, Origin, Pass, author};
use crate::template::{GitRunner, RealGit};
use crate::workspace::repo_git;
use std::io;
use std::path::Path;

/// Does `config/<name>` exist in the workspace repository?
fn has_ref(ws: &Path, name: &str) -> bool {
    RealGit::new()
        .run(
            &repo_git(ws),
            &[
                "show-ref",
                "--verify",
                "--quiet",
                &format!("refs/heads/config/{name}"),
            ],
        )
        .is_ok()
}

#[test]
fn a_no_op_edit_is_a_declined_pass_not_a_failure() {
    let (holder, ws) = workspace();
    // An advance whose edit writes nothing leaves the checkout identical
    // to the branch head (no template extract on advance, empty pool):
    // nothing is staged, so nothing is authored — and that is a success.
    let head = show(&ws, "config/default:version").unwrap();
    let pass = author(
        &ws,
        &holder.path().join("no-pool"),
        "default",
        Origin::Advance,
        write_files(&[]),
        &RealGit::new(),
    )
    .unwrap();
    assert_eq!(
        pass,
        Pass::Declined {
            target: "config/default".to_string()
        }
    );
    assert_eq!(show(&ws, "config/default:version").unwrap(), head);
    assert!(!ws.join(".config-author").exists(), "checkout must be gone");
}

#[test]
fn a_declined_pass_leaves_the_verb_ready_for_an_immediate_re_author() {
    let (holder, ws) = workspace();
    let no_pool = holder.path().join("no-pool");
    author(
        &ws,
        &no_pool,
        "default",
        Origin::Advance,
        write_files(&[]),
        &RealGit::new(),
    )
    .unwrap();
    // The next pass materializes on the same path and lands: the decline
    // wedged nothing.
    let pass = author(
        &ws,
        &no_pool,
        "default",
        Origin::Advance,
        write_files(&[("providers.yaml", "roles: {second: 1}\n")]),
        &RealGit::new(),
    )
    .unwrap();
    assert_eq!(pass, Pass::Landed);
    assert_eq!(
        show(&ws, "config/default:providers.yaml").unwrap(),
        "roles: {second: 1}"
    );
}

#[test]
fn a_declined_fork_leaves_no_ref() {
    let (holder, ws) = workspace();
    let pass = author(
        &ws,
        &holder.path().join("no-pool"),
        "stillborn",
        Origin::Fork { source: "default" },
        write_files(&[]),
        &RealGit::new(),
    )
    .unwrap();
    assert!(matches!(pass, Pass::Declined { .. }), "got {pass:?}");
    assert!(!has_ref(&ws, "stillborn"), "declined fork left a ref");
    assert!(!ws.join(".config-author").exists());
}

#[test]
fn a_failed_fork_leaves_neither_checkout_nor_ref() {
    let (holder, ws) = workspace();
    // The edit step blows up after the branch was created by the
    // `worktree add -b`; teardown still runs, ref included.
    let err = author(
        &ws,
        &holder.path().join("no-pool"),
        "doomed",
        Origin::Fork { source: "default" },
        |_dir: &Path| Err(io::Error::other("editor blew up")),
        &RealGit::new(),
    )
    .unwrap_err();
    assert!(matches!(err, Error::Edit(_)), "got {err:?}");
    assert!(!has_ref(&ws, "doomed"), "failed fork left a ref");
    assert!(!ws.join(".config-author").exists());
}

#[test]
fn a_failed_orphan_leaves_neither_checkout_nor_ref() {
    let (holder, ws) = workspace();
    let err = author(
        &ws,
        &holder.path().join("no-pool"),
        "doomed",
        Origin::Orphan,
        |_dir: &Path| Err(io::Error::other("editor blew up")),
        &RealGit::new(),
    )
    .unwrap_err();
    assert!(matches!(err, Error::Edit(_)), "got {err:?}");
    assert!(!has_ref(&ws, "doomed"), "failed orphan left a ref");
}

#[test]
fn a_killed_pass_leaves_debris_the_next_pass_heals() {
    let (holder, ws) = workspace();
    let no_pool = holder.path().join("no-pool");
    // Exactly what a SIGKILL mid-pass leaves: the checkout registered as
    // a worktree, dirty, with no teardown ever run.
    RealGit::new()
        .run(
            &repo_git(&ws),
            &[
                "worktree",
                "add",
                &ws.join(".config-author").to_string_lossy(),
                "config/default",
            ],
        )
        .unwrap();
    std::fs::write(ws.join(".config-author/half-typed.yaml"), "oops").unwrap();

    let pass = author(
        &ws,
        &no_pool,
        "default",
        Origin::Advance,
        write_files(&[("providers.yaml", "roles: {healed: 1}\n")]),
        &RealGit::new(),
    )
    .unwrap();
    assert_eq!(pass, Pass::Landed);
    // The killed pass's unsaved edit is gone with its checkout, and the
    // new commit carries only this pass's edit.
    assert_eq!(
        show(&ws, "config/default:providers.yaml").unwrap(),
        "roles: {healed: 1}"
    );
    assert!(show(&ws, "config/default:half-typed.yaml").is_err());
    assert!(!ws.join(".config-author").exists());
}

#[test]
fn an_unremovable_checkout_path_surfaces_rather_than_wedging_silently() {
    let (holder, ws) = workspace();
    // A regular file squatting the checkout path is not a worktree git
    // can remove; `remove_dir_all` refuses it too, so the heal reports.
    std::fs::write(ws.join(".config-author"), b"squat").unwrap();
    let err = author(
        &ws,
        &holder.path().join("no-pool"),
        "default",
        Origin::Advance,
        write_files(&[]),
        &RealGit::new(),
    )
    .unwrap_err();
    assert!(matches!(err, Error::Io(_)), "got {err:?}");
}
