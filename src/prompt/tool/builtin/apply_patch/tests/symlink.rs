//! The write-time symlink discipline at destinations (bl-91f8 idiom,
//! bl-2502): add, update, and rename targets that are themselves
//! symlinks — dangling or not — are declined before anything is
//! staged; delete acts on the link itself and needs no guard.

use super::super::apply::{Error, apply};
use super::parsed;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use tempfile::TempDir;

fn err_for(body: &str, root: &Path) -> Error {
    apply(&parsed(body), root).unwrap_err()
}

fn assert_symlink_decline(err: &Error, want_action: &str) {
    match err {
        Error::SymlinkDest { action, .. } => assert_eq!(*action, want_action),
        other => panic!("expected a symlink decline, got {other}"),
    }
}

// bl-2502: `exists()` follows links, so a dangling symlink read as a
// vacant path — the add guard passed and the write landed through the
// link, outside the authored path.
#[test]
fn add_over_a_dangling_symlink_is_declined_and_writes_nothing() {
    let tmp = TempDir::new().unwrap();
    let outside = tmp.path().join("outside.txt");
    symlink(&outside, tmp.path().join("link.txt")).unwrap();
    let err = err_for("*** Add File: link.txt\n+smuggled", tmp.path());
    assert_symlink_decline(&err, "add");
    assert!(
        err.to_string().contains("destination is a symlink"),
        "{err}"
    );
    assert!(!outside.exists(), "no bytes may land through the link");
}

#[test]
fn add_over_a_live_symlink_is_declined_as_a_symlink_not_as_existing() {
    let tmp = TempDir::new().unwrap();
    let target = tmp.path().join("real.txt");
    fs::write(&target, "kept\n").unwrap();
    symlink(&target, tmp.path().join("link.txt")).unwrap();
    let err = err_for("*** Add File: link.txt\n+clobber", tmp.path());
    assert_symlink_decline(&err, "add");
    assert_eq!(fs::read_to_string(&target).unwrap(), "kept\n");
}

// `fs::write` on an update follows the link too: the patch names the
// link's path but the bytes would land at its target.
#[test]
fn update_through_a_symlink_is_declined() {
    let tmp = TempDir::new().unwrap();
    let target = tmp.path().join("real.txt");
    fs::write(&target, "x\n").unwrap();
    symlink(&target, tmp.path().join("link.txt")).unwrap();
    let err = err_for("*** Update File: link.txt\n-x\n+y", tmp.path());
    assert_symlink_decline(&err, "update");
    assert_eq!(fs::read_to_string(&target).unwrap(), "x\n");
}

// A dangling symlink at a rename destination also read as vacant to
// the `exists()` guard; the destination is occupied by the link.
#[test]
fn rename_onto_a_dangling_symlink_is_declined() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("a.txt"), "x\n").unwrap();
    symlink(tmp.path().join("gone"), tmp.path().join("link.txt")).unwrap();
    let err = err_for(
        "*** Update File: a.txt\n*** Move to: link.txt\n-x\n+y",
        tmp.path(),
    );
    assert_symlink_decline(&err, "move to");
    assert_eq!(fs::read_to_string(tmp.path().join("a.txt")).unwrap(), "x\n");
}

// Delete is exempt from the guard: `fs::remove_file` acts on the link
// itself, so only the link goes and the target keeps its bytes.
#[test]
fn delete_of_a_symlink_removes_the_link_and_keeps_the_target() {
    let tmp = TempDir::new().unwrap();
    let target = tmp.path().join("real.txt");
    fs::write(&target, "kept\n").unwrap();
    let link = tmp.path().join("link.txt");
    symlink(&target, &link).unwrap();
    apply(&parsed("*** Delete File: link.txt"), tmp.path()).unwrap();
    assert!(fs::symlink_metadata(&link).is_err(), "link is gone");
    assert_eq!(fs::read_to_string(&target).unwrap(), "kept\n");
}
