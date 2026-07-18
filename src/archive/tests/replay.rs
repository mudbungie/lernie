//! `replay` cases (ARCH §9.2): scratch reconstruction, slice restoration,
//! and every error arm. The shared stub and fixtures live in the parent
//! [`super`] module; the bundle-heads fixtures are local to replay.

use super::super::{ArchiveError, BUNDLE_FILE, replay};
use super::{AGENT, StubGit, tmp, write};
use std::fs;

/// Canned `git bundle list-heads` output for the two-branch subtree,
/// including a short line to exercise the parse filter.
const HEADS: &str =
    "sha1 refs/heads/agents/20260101-p1\nsha2 refs/heads/agents/20260101-p1-20260102-c1\n\n";

/// An archive dir with a (touched) bundle file plus both slices.
fn archive_with_bundle() -> tempfile::TempDir {
    let arch = tmp();
    fs::write(arch.path().join(BUNDLE_FILE), b"").unwrap();
    write(&arch.path().join("steps/20260101-p1/001/meta.json"), "{}");
    write(&arch.path().join("inbox/20260101-p1/user-001.md"), "hi");
    arch
}

#[test]
fn replay_reconstructs_scratch_and_restores_slices() {
    let arch = archive_with_bundle();
    let base = tmp();
    let git = StubGit::new(HEADS);

    let scratch = replay(arch.path(), base.path(), &git).unwrap();

    assert_eq!(scratch, base.path().join(AGENT));
    // init, fetch, worktree add — in order.
    let runs = git.runs.borrow();
    assert_eq!(runs[0][0], "init");
    assert_eq!(runs[1][0], "fetch");
    assert_eq!(runs[2][0], "worktree");
    // Slices restored under the scratch workspace.
    assert!(scratch.join("steps/20260101-p1/001/meta.json").exists());
    assert!(scratch.join("inbox/20260101-p1/user-001.md").exists());
}

#[test]
fn replay_rejects_missing_bundle() {
    let arch = tmp();
    let base = tmp();
    let git = StubGit::new(HEADS);
    let err = replay(arch.path(), base.path(), &git).unwrap_err();
    assert!(matches!(err, ArchiveError::BundleMissing(_)), "{err:?}");
}

#[test]
fn replay_rejects_empty_bundle() {
    let arch = archive_with_bundle();
    let base = tmp();
    let git = StubGit::new(""); // list-heads names nothing
    let err = replay(arch.path(), base.path(), &git).unwrap_err();
    assert!(matches!(err, ArchiveError::EmptyBundle), "{err:?}");
}

#[test]
fn replay_rejects_malformed_bundle() {
    let arch = archive_with_bundle();
    let base = tmp();
    // Two heads sharing no common subtree root.
    let git = StubGit::new("s1 refs/heads/agents/aa-b\ns2 refs/heads/agents/xx-y\n");
    let err = replay(arch.path(), base.path(), &git).unwrap_err();
    assert!(matches!(err, ArchiveError::MalformedBundle(_)), "{err:?}");
}

#[test]
fn replay_rejects_existing_destination() {
    let arch = archive_with_bundle();
    let base = tmp();
    fs::create_dir_all(base.path().join(AGENT)).unwrap();
    let git = StubGit::new(HEADS);
    let err = replay(arch.path(), base.path(), &git).unwrap_err();
    assert!(matches!(err, ArchiveError::DestExists(_)), "{err:?}");
}

#[test]
fn replay_surfaces_list_heads_failure() {
    let arch = archive_with_bundle();
    let base = tmp();
    let git = StubGit::new(HEADS).fail_capture();
    let err = replay(arch.path(), base.path(), &git).unwrap_err();
    assert!(
        matches!(
            err,
            ArchiveError::Git {
                op: "bundle list-heads",
                ..
            }
        ),
        "{err:?}"
    );
}

#[test]
fn replay_surfaces_init_fetch_worktree_failures() {
    for (idx, op) in [(0usize, "init"), (1, "fetch"), (2, "worktree add")] {
        let arch = archive_with_bundle();
        let base = tmp();
        let git = StubGit::new(HEADS).fail_run_at(idx);
        let err = replay(arch.path(), base.path(), &git).unwrap_err();
        match err {
            ArchiveError::Git { op: got, .. } => assert_eq!(got, op),
            other => panic!("expected git {op} failure, got {other:?}"),
        }
    }
}

#[test]
fn replay_tolerates_absent_slices() {
    // Bundle present, but no steps/inbox dirs to restore.
    let arch = tmp();
    fs::write(arch.path().join(BUNDLE_FILE), b"").unwrap();
    let base = tmp();
    let git = StubGit::new(HEADS);
    let scratch = replay(arch.path(), base.path(), &git).unwrap();
    assert!(!scratch.join("steps").exists());
    assert!(!scratch.join("inbox").exists());
}
