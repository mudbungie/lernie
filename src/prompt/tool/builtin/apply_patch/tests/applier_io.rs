//! Write-phase I/O faults: the staged patch was valid, the filesystem
//! then refused. Each test pins the action word the decline leads with,
//! and restores permissions so the tempdir can be reaped.

use super::super::apply::{Error, apply};
use super::parsed;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tempfile::TempDir;

fn chmod(p: &Path, mode: u32) {
    fs::set_permissions(p, fs::Permissions::from_mode(mode)).unwrap();
}

fn err_for(body: &str, root: &Path) -> Error {
    apply(&parsed(body), root).unwrap_err()
}

fn action_of(err: &Error) -> &'static str {
    match err {
        Error::Io { action, .. } => action,
        other => panic!("expected an Io fault, got {other}"),
    }
}

#[test]
fn a_file_standing_where_a_directory_is_needed_fails_directory_creation() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("blocker"), "a file\n").unwrap();
    let err = err_for("*** Add File: blocker/inner.txt\n+x", tmp.path());
    assert_eq!(action_of(&err), "create directory for");
}

#[test]
fn an_unwritable_directory_fails_the_add_write() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().join("ro");
    fs::create_dir(&dir).unwrap();
    chmod(&dir, 0o555);
    let err = err_for("*** Add File: ro/new.txt\n+x", tmp.path());
    chmod(&dir, 0o755);
    assert_eq!(action_of(&err), "write");
}

#[test]
fn an_unwritable_file_fails_the_update_write() {
    // Reading a read-only file stages fine; reopening it for the write
    // is what the kernel refuses.
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("a.txt");
    fs::write(&file, "x\n").unwrap();
    chmod(&file, 0o444);
    let err = err_for("*** Update File: a.txt\n-x\n+y", tmp.path());
    chmod(&file, 0o644);
    assert_eq!(action_of(&err), "write");
}

#[test]
fn an_unwritable_directory_fails_the_delete() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().join("ro");
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("a.txt"), "x\n").unwrap();
    chmod(&dir, 0o555);
    let err = err_for("*** Delete File: ro/a.txt", tmp.path());
    chmod(&dir, 0o755);
    assert_eq!(action_of(&err), "delete");
}

#[test]
fn an_unwritable_destination_directory_fails_the_rename() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("a.txt"), "x\n").unwrap();
    let dir = tmp.path().join("ro");
    fs::create_dir(&dir).unwrap();
    chmod(&dir, 0o555);
    let err = err_for(
        "*** Update File: a.txt\n*** Move to: ro/b.txt\n-x\n+y",
        tmp.path(),
    );
    chmod(&dir, 0o755);
    assert_eq!(action_of(&err), "rename");
}

#[test]
fn an_uncreatable_destination_directory_fails_before_the_rename() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("a.txt"), "x\n").unwrap();
    let dir = tmp.path().join("ro");
    fs::create_dir(&dir).unwrap();
    chmod(&dir, 0o555);
    let err = err_for(
        "*** Update File: a.txt\n*** Move to: ro/deeper/b.txt\n-x\n+y",
        tmp.path(),
    );
    chmod(&dir, 0o755);
    assert_eq!(action_of(&err), "create directory for");
}
