use super::*;
use std::os::unix::fs::symlink;
use tempfile::TempDir;

fn write(p: &Path, body: &str) {
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(p, body).unwrap();
}

/// Lay out a synthetic `/proc/<pid>/{fd,fdinfo,stat}` plus a
/// real on-disk file so `canonicalize` resolves cleanly. Returns
/// (proc_root, response_path).
fn fixture(pid: i32, fd: u32, flags: &str, pgid: i32) -> (TempDir, std::path::PathBuf) {
    let dir = TempDir::new().unwrap();
    let response = dir.path().join("response.json");
    write(&response, "");
    let proc_root = dir.path().join("proc");
    let fd_dir = proc_root.join(pid.to_string()).join("fd");
    std::fs::create_dir_all(&fd_dir).unwrap();
    symlink(&response, fd_dir.join(fd.to_string())).unwrap();
    write(
        &proc_root
            .join(pid.to_string())
            .join("fdinfo")
            .join(fd.to_string()),
        &format!("pos:\t0\nflags:\t{flags}\nmnt_id:\t1\n"),
    );
    write(
        &proc_root.join(pid.to_string()).join("stat"),
        &format!("{pid} (lernie) S 1 {pgid} 0 0 -1 0 0 0 0 0 0 0 0 0 20 0 1 0\n"),
    );
    (dir, response)
}

#[test]
fn finder_returns_none_when_response_path_missing() {
    let dir = TempDir::new().unwrap();
    let f = ProcFsFinder::with_root(dir.path().join("proc-empty"));
    let result = f
        .find_writer_pgid(&dir.path().join("missing.json"))
        .unwrap();
    assert!(result.is_none());
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
    let through_file = regular.join("inside").join("response.json");
    let f = ProcFsFinder::with_root(dir.path().join("proc-empty"));
    let err = f.find_writer_pgid(&through_file).unwrap_err();
    assert_ne!(err.kind(), io::ErrorKind::NotFound);
}

#[test]
fn finder_returns_pgid_for_writable_fd() {
    let (dir, response) = fixture(1234, 7, "0102001", 9999);
    let f = ProcFsFinder::with_root(dir.path().join("proc"));
    let pgid = f.find_writer_pgid(&response).unwrap();
    assert_eq!(pgid, Some(9999));
}

#[test]
fn finder_skips_readonly_fd() {
    let (dir, response) = fixture(1234, 7, "0100000", 9999);
    let f = ProcFsFinder::with_root(dir.path().join("proc"));
    let pgid = f.find_writer_pgid(&response).unwrap();
    assert!(pgid.is_none());
}

#[test]
fn finder_skips_unparseable_pid_dirs() {
    let (dir, response) = fixture(1234, 7, "0102001", 9999);
    // Add a non-numeric dir like `/proc/self` or `/proc/sys`.
    std::fs::create_dir_all(dir.path().join("proc").join("sys")).unwrap();
    std::fs::create_dir_all(dir.path().join("proc").join("self")).unwrap();
    let f = ProcFsFinder::with_root(dir.path().join("proc"));
    let pgid = f.find_writer_pgid(&response).unwrap();
    assert_eq!(pgid, Some(9999));
}

#[test]
fn finder_skips_fd_with_unrelated_target() {
    let (dir, response) = fixture(1234, 7, "0102001", 9999);
    // Add a second pid whose fd points elsewhere — must not match.
    let other_target = dir.path().join("other.json");
    write(&other_target, "");
    let other_fd_dir = dir.path().join("proc").join("5555").join("fd");
    std::fs::create_dir_all(&other_fd_dir).unwrap();
    symlink(&other_target, other_fd_dir.join("3")).unwrap();
    let f = ProcFsFinder::with_root(dir.path().join("proc"));
    let pgid = f.find_writer_pgid(&response).unwrap();
    assert_eq!(pgid, Some(9999));
}

#[test]
fn finder_skips_pid_dir_with_no_fd_subdir() {
    let (dir, response) = fixture(1234, 7, "0102001", 9999);
    // Pid with stat but no fd dir → unread; not a match.
    std::fs::create_dir_all(dir.path().join("proc").join("7777")).unwrap();
    let f = ProcFsFinder::with_root(dir.path().join("proc"));
    let pgid = f.find_writer_pgid(&response).unwrap();
    assert_eq!(pgid, Some(9999));
}

#[test]
fn finder_handles_missing_fdinfo_as_not_writable() {
    let (dir, response) = fixture(1234, 7, "0102001", 9999);
    std::fs::remove_file(
        dir.path()
            .join("proc")
            .join("1234")
            .join("fdinfo")
            .join("7"),
    )
    .unwrap();
    let f = ProcFsFinder::with_root(dir.path().join("proc"));
    let pgid = f.find_writer_pgid(&response).unwrap();
    assert!(pgid.is_none());
}

#[test]
fn finder_handles_empty_flags_line_as_not_writable() {
    let (dir, response) = fixture(1234, 7, "", 9999);
    let f = ProcFsFinder::with_root(dir.path().join("proc"));
    let pgid = f.find_writer_pgid(&response).unwrap();
    assert!(pgid.is_none());
}

#[test]
fn finder_handles_fdinfo_without_flags_line_as_not_writable() {
    let (dir, response) = fixture(1234, 7, "0102001", 9999);
    std::fs::write(
        dir.path()
            .join("proc")
            .join("1234")
            .join("fdinfo")
            .join("7"),
        "pos:\t0\nmnt_id:\t1\n",
    )
    .unwrap();
    let f = ProcFsFinder::with_root(dir.path().join("proc"));
    let pgid = f.find_writer_pgid(&response).unwrap();
    assert!(pgid.is_none());
}

#[test]
fn read_pgid_errors_on_malformed_stat_no_close_paren() {
    let dir = TempDir::new().unwrap();
    write(
        &dir.path().join("proc").join("1234").join("stat"),
        "1234 lernie S 1 9999 0\n",
    );
    let err = read_pgid(&dir.path().join("proc"), 1234, 0).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
}

#[test]
fn read_pgid_errors_on_truncated_fields() {
    let dir = TempDir::new().unwrap();
    write(
        &dir.path().join("proc").join("1234").join("stat"),
        "1234 (lernie) S 1\n",
    );
    let err = read_pgid(&dir.path().join("proc"), 1234, 0).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
}

#[test]
fn read_pgid_errors_on_unparseable_pgid() {
    let dir = TempDir::new().unwrap();
    write(
        &dir.path().join("proc").join("1234").join("stat"),
        "1234 (lernie) S 1 not-a-number 0\n",
    );
    let err = read_pgid(&dir.path().join("proc"), 1234, 0).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
}

#[test]
fn finder_treats_fd_with_unparseable_name_as_skip() {
    let (dir, response) = fixture(1234, 7, "0102001", 9999);
    // Add a non-numeric fd entry.
    let bogus_target = dir.path().join("response.json");
    symlink(
        &bogus_target,
        dir.path()
            .join("proc")
            .join("1234")
            .join("fd")
            .join("notanum"),
    )
    .unwrap();
    let f = ProcFsFinder::with_root(dir.path().join("proc"));
    let pgid = f.find_writer_pgid(&response).unwrap();
    assert_eq!(pgid, Some(9999));
}
