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

/// Copy each entry of `src_root` named `<agent_id>` or `<agent_id>-*`
/// into `dst_root`. A missing `src_root` (no slice for this run) is a
/// clean no-op; `dst_root` is created only when something matches.
pub(super) fn copy_matching(src_root: &Path, dst_root: &Path, agent_id: &str) -> io::Result<()> {
    if !src_root.is_dir() {
        return Ok(());
    }
    let prefix = format!("{agent_id}-");
    for entry in fs::read_dir(src_root)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_s = name.to_string_lossy();
        if *name_s == *agent_id || name_s.starts_with(&prefix) {
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
