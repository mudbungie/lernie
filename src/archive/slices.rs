//! The archive's non-git half: the `steps/<id>*` and `inbox/<id>*`
//! diagnostic slices that ride beside the bundle (ARCH §9.2 "One bundle
//! plus two slices").
//!
//! These are plain directories outside git (§2.2), so archiving and
//! restoring them is directory copying and nothing more — kept here so
//! [`super`] holds only the ref-and-ancestry logic.

use std::fs;
use std::io;
use std::path::Path;

/// The diagnostic slice directory names carried beside the bundle (§2.2).
pub(super) const SLICES: [&str; 2] = ["steps", "inbox"];

/// Is `name` inside the subtree rooted at `agent_id` — the id itself or
/// one of its `<agent_id>-*` hyphen-descendants (§2.3)? The one home of
/// the descent-prefix test, shared by the archive's slice copy and the
/// retention delete ([`super::delete`]), so both cut the subtree on the
/// same line.
pub(super) fn in_subtree(name: &str, agent_id: &str) -> bool {
    name == agent_id || name.starts_with(&format!("{agent_id}-"))
}

/// Copy each entry of `src_root` named `<agent_id>` or `<agent_id>-*`
/// into `dst_root`. A missing `src_root` (no slice for this run) is a
/// clean no-op; `dst_root` is created only when something matches.
pub(super) fn copy_matching(src_root: &Path, dst_root: &Path, agent_id: &str) -> io::Result<()> {
    if !src_root.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(src_root)? {
        let entry = entry?;
        let name = entry.file_name();
        if in_subtree(&name.to_string_lossy(), agent_id) {
            copy_entry(&entry.path(), &dst_root.join(&name))?;
        }
    }
    Ok(())
}

/// Recursively copy a directory tree.
pub(super) fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        copy_entry(&entry.path(), &dst.join(entry.file_name()))?;
    }
    Ok(())
}

/// Copy one filesystem entry — recursing for directories, `fs::copy` for
/// files.
fn copy_entry(src: &Path, dst: &Path) -> io::Result<()> {
    if src.is_dir() {
        copy_dir_all(src, dst)
    } else {
        fs::copy(src, dst).map(|_| ())
    }
}
