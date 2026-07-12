//! Executor-lock tests (ARCH §2.11 *The executor lock*).

use super::super::lock::{interpret_lock, lock_or_none, try_acquire};
use std::io;
use tempfile::TempDir;

#[test]
fn acquire_creates_inbox_and_returns_guard() {
    let ws = TempDir::new().unwrap();
    let inbox = ws.path().join("inbox").join("a1");
    assert!(!inbox.exists());
    let guard = try_acquire(&inbox).unwrap();
    assert!(guard.is_some(), "fresh inbox is acquirable");
    assert!(inbox.is_dir(), "acquire creates the inbox home");
}

#[test]
fn second_acquire_is_excluded_while_first_held() {
    let ws = TempDir::new().unwrap();
    let inbox = ws.path().join("inbox").join("a1");
    let held = try_acquire(&inbox).unwrap();
    assert!(held.is_some());
    // A separate open file description on the same directory contends
    // even inside one process (flock(2)); the second probe sees None.
    let second = try_acquire(&inbox).unwrap();
    assert!(second.is_none(), "held lock excludes a second driver");
}

#[test]
fn lock_releases_on_drop() {
    let ws = TempDir::new().unwrap();
    let inbox = ws.path().join("inbox").join("a1");
    let first = try_acquire(&inbox).unwrap().expect("acquirable");
    drop(first); // release the lease
    let again = try_acquire(&inbox).unwrap();
    assert!(again.is_some(), "dropping the guard frees the lock");
}

#[test]
fn acquire_surfaces_io_error_when_inbox_home_blocked() {
    let ws = TempDir::new().unwrap();
    // Make `<ws>/inbox` a regular file so create_dir_all of
    // `<ws>/inbox/a1` fails — the open-inbox `?` error arm.
    std::fs::write(ws.path().join("inbox"), b"not a dir").unwrap();
    let inbox = ws.path().join("inbox").join("a1");
    let err = try_acquire(&inbox).unwrap_err();
    assert!(
        matches!(
            err.kind(),
            io::ErrorKind::AlreadyExists | io::ErrorKind::NotADirectory | io::ErrorKind::Other
        ),
        "unexpected kind: {err:?}"
    );
}

#[test]
fn interpret_lock_maps_success_to_guard() {
    let f = tempfile::tempfile().unwrap();
    let out = interpret_lock(f, 0, io::Error::from_raw_os_error(0)).unwrap();
    assert!(out.is_some(), "ret==0 is a held lease");
}

#[test]
fn interpret_lock_maps_would_block_to_none() {
    let f = tempfile::tempfile().unwrap();
    let err = io::Error::from_raw_os_error(libc::EWOULDBLOCK);
    let out = interpret_lock(f, -1, err).unwrap();
    assert!(out.is_none(), "EWOULDBLOCK means someone else drives");
}

#[test]
fn interpret_lock_propagates_other_errno() {
    let f = tempfile::tempfile().unwrap();
    let err = io::Error::from_raw_os_error(libc::EBADF);
    let out = interpret_lock(f, -1, err);
    assert!(out.is_err(), "an unexpected errno propagates");
}

#[test]
fn lock_or_none_locks_a_real_fd() {
    // Drives the production syscall path (not the pure interpreter):
    // a fresh unlinked temp file is unlocked, so this acquires.
    let f = tempfile::tempfile().unwrap();
    let out = lock_or_none(f).unwrap();
    assert!(out.is_some());
}
