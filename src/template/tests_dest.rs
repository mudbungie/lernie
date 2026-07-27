//! Unit tests for `check_dest` — the guard `scaffold` runs first
//! (ARCH §2.2): refuse a non-empty directory or an existing
//! non-directory path in the same voice; accept a missing path or an
//! empty directory.

use super::*;
use tempfile::TempDir;

#[test]
fn check_dest_allows_missing_path() {
    let holder = TempDir::new().unwrap();
    let missing = holder.path().join("nope");
    check_dest(&missing).unwrap();
}

#[test]
fn check_dest_allows_empty_directory() {
    let holder = TempDir::new().unwrap();
    check_dest(holder.path()).unwrap();
}

#[test]
fn check_dest_rejects_non_empty_directory() {
    let holder = TempDir::new().unwrap();
    fs::write(holder.path().join("occupant"), b"x").unwrap();
    let err = check_dest(holder.path()).unwrap_err();
    assert!(matches!(err, ScaffoldError::DestNotEmpty(_)));
}

#[test]
fn check_dest_rejects_a_plain_file() {
    // read_dir on a regular file fails with NotADirectory: the same
    // guard as the non-empty-directory case, one more condition.
    let holder = TempDir::new().unwrap();
    let file = holder.path().join("actually-a-file");
    fs::write(&file, b"not a dir").unwrap();
    let err = check_dest(&file).unwrap_err();
    assert!(matches!(err, ScaffoldError::DestNotDir(_)), "got {err:?}");
    assert_eq!(
        err.to_string(),
        format!(
            "destination {} already exists and is not a directory",
            file.display()
        )
    );
}
