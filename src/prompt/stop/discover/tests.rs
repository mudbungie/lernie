use super::*;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use tempfile::TempDir;

fn write(p: &Path, body: &str) {
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(p, body).unwrap();
}

/// Lay out a synthetic `/proc/<pid>/{fd,stat}` plus a real on-disk inbox
/// directory so `canonicalize` resolves cleanly and `<pid>/fd/<fd>`
/// symlinks to it — the executor-lock fd (§2.11). Returns (tmp,
/// inbox_dir).
fn fixture(pid: i32, fd: u32, pgid: i32) -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let inbox = dir.path().join("inbox").join("agent-1");
    std::fs::create_dir_all(&inbox).unwrap();
    let proc_root = dir.path().join("proc");
    let fd_dir = proc_root.join(pid.to_string()).join("fd");
    std::fs::create_dir_all(&fd_dir).unwrap();
    symlink(&inbox, fd_dir.join(fd.to_string())).unwrap();
    write(
        &proc_root.join(pid.to_string()).join("stat"),
        &format!("{pid} (lernie) S 1 {pgid} 0 0 -1 0 0 0 0 0 0 0 0 0 20 0 1 0\n"),
    );
    (dir, inbox)
}

#[test]
fn finder_returns_none_when_inbox_dir_missing() {
    let dir = TempDir::new().unwrap();
    let f = ProcFsFinder::with_root(dir.path().join("proc-empty"));
    let result = f.find_holder_pgid(&dir.path().join("missing")).unwrap();
    assert!(result.is_none());
}

#[test]
fn finder_returns_none_when_no_pid_holds_the_inbox() {
    // The inbox dir exists (canonicalize succeeds) but the sole pid in
    // the fixture holds an fd on an unrelated path — the scan exhausts
    // without a match and returns None (the "already-stopped" path).
    let dir = TempDir::new().unwrap();
    let inbox = dir.path().join("inbox").join("agent-1");
    std::fs::create_dir_all(&inbox).unwrap();
    let elsewhere = dir.path().join("elsewhere");
    std::fs::create_dir_all(&elsewhere).unwrap();
    let fd_dir = dir.path().join("proc").join("5555").join("fd");
    std::fs::create_dir_all(&fd_dir).unwrap();
    symlink(&elsewhere, fd_dir.join("3")).unwrap();
    let f = ProcFsFinder::with_root(dir.path().join("proc"));
    assert!(f.find_holder_pgid(&inbox).unwrap().is_none());
}

#[test]
fn finder_default_points_at_proc() {
    // Default impl is what production uses. Smoke-check the
    // root, not actual scanning — `/proc` is real on the host
    // but we don't want a unit test to depend on its contents.
    let f = ProcFsFinder::default();
    assert_eq!(f.proc_root, std::path::PathBuf::from("/proc"));
}

#[test]
fn finder_propagates_canonicalize_error_other_than_not_found() {
    // Canonicalize on a path whose intermediate component is a
    // regular file (not a directory) returns NotADirectory, not
    // NotFound — exercises the catch-all error branch.
    let dir = TempDir::new().unwrap();
    let regular = dir.path().join("regular");
    std::fs::write(&regular, "x").unwrap();
    let through_file = regular.join("inside").join("inbox");
    let f = ProcFsFinder::with_root(dir.path().join("proc-empty"));
    let err = f.find_holder_pgid(&through_file).unwrap_err();
    assert_ne!(err.kind(), io::ErrorKind::NotFound);
}

#[test]
fn finder_returns_pgid_for_inbox_fd_holder() {
    let (dir, inbox) = fixture(1234, 7, 9999);
    let f = ProcFsFinder::with_root(dir.path().join("proc"));
    let pgid = f.find_holder_pgid(&inbox).unwrap();
    assert_eq!(pgid, Some(9999));
}

#[test]
fn finder_skips_unparseable_pid_dirs() {
    let (dir, inbox) = fixture(1234, 7, 9999);
    // Add non-numeric dirs like `/proc/self` or `/proc/sys`.
    std::fs::create_dir_all(dir.path().join("proc").join("sys")).unwrap();
    std::fs::create_dir_all(dir.path().join("proc").join("self")).unwrap();
    let f = ProcFsFinder::with_root(dir.path().join("proc"));
    let pgid = f.find_holder_pgid(&inbox).unwrap();
    assert_eq!(pgid, Some(9999));
}

#[test]
fn finder_skips_fd_with_unrelated_target() {
    let (dir, inbox) = fixture(1234, 7, 9999);
    // A second pid whose fd points elsewhere — must not match.
    let other_target = dir.path().join("other");
    std::fs::create_dir_all(&other_target).unwrap();
    let other_fd_dir = dir.path().join("proc").join("5555").join("fd");
    std::fs::create_dir_all(&other_fd_dir).unwrap();
    symlink(&other_target, other_fd_dir.join("3")).unwrap();
    let f = ProcFsFinder::with_root(dir.path().join("proc"));
    let pgid = f.find_holder_pgid(&inbox).unwrap();
    assert_eq!(pgid, Some(9999));
}

#[test]
fn finder_skips_pid_dir_with_no_fd_subdir() {
    let (dir, inbox) = fixture(1234, 7, 9999);
    // Pid with stat but no fd dir → unread; not a match.
    std::fs::create_dir_all(dir.path().join("proc").join("7777")).unwrap();
    let f = ProcFsFinder::with_root(dir.path().join("proc"));
    let pgid = f.find_holder_pgid(&inbox).unwrap();
    assert_eq!(pgid, Some(9999));
}

#[test]
fn finder_skips_non_symlink_fd_entry() {
    let (dir, inbox) = fixture(1234, 7, 9999);
    // A regular file under another pid's fd dir: `read_link` errors
    // (not a symlink) → the entry is skipped rather than fatal.
    let other_fd_dir = dir.path().join("proc").join("6666").join("fd");
    std::fs::create_dir_all(&other_fd_dir).unwrap();
    std::fs::write(other_fd_dir.join("0"), "not a symlink").unwrap();
    let f = ProcFsFinder::with_root(dir.path().join("proc"));
    let pgid = f.find_holder_pgid(&inbox).unwrap();
    assert_eq!(pgid, Some(9999));
}

#[test]
fn read_pgid_errors_on_malformed_stat_no_close_paren() {
    let dir = TempDir::new().unwrap();
    write(
        &dir.path().join("proc").join("1234").join("stat"),
        "1234 lernie S 1 9999 0\n",
    );
    let err = read_pgid(&dir.path().join("proc"), 1234).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
}

#[test]
fn read_pgid_errors_on_truncated_fields() {
    let dir = TempDir::new().unwrap();
    write(
        &dir.path().join("proc").join("1234").join("stat"),
        "1234 (lernie) S 1\n",
    );
    let err = read_pgid(&dir.path().join("proc"), 1234).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
}

#[test]
fn read_pgid_errors_on_unparseable_pgid() {
    let dir = TempDir::new().unwrap();
    write(
        &dir.path().join("proc").join("1234").join("stat"),
        "1234 (lernie) S 1 not-a-number 0\n",
    );
    let err = read_pgid(&dir.path().join("proc"), 1234).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
}

#[test]
fn finder_treats_fd_with_unparseable_name_as_skip() {
    let (dir, inbox) = fixture(1234, 7, 9999);
    // A non-numeric fd entry still symlinks to the inbox; the scan
    // matches on the target, not the fd name, so it is found via the
    // numeric fd 7 regardless.
    symlink(
        &inbox,
        dir.path()
            .join("proc")
            .join("1234")
            .join("fd")
            .join("notanum"),
    )
    .unwrap();
    let f = ProcFsFinder::with_root(dir.path().join("proc"));
    let pgid = f.find_holder_pgid(&inbox).unwrap();
    assert_eq!(pgid, Some(9999));
}
