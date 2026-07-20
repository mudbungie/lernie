//! The one interactive sliver of the exec binding that cannot be
//! unit-tested and so lives at the coverage-exempt bin seam (ARCH §3.4):
//! the `lernie config` `$EDITOR` hand-off, injected as
//! [`lernie::cmd::Fx::editor`]. Everything testable — origin resolution
//! and the commit — lives in the crate's private `template::authoring`
//! machinery; only the interactive spawn is here. The `lernie advance` successor
//! `exec` is no longer a bespoke handler: it rides the generic
//! [`lernie::cmd::Outcome::Exec`] the binding performs in `main`.

use std::io;
use std::path::Path;

/// The `lernie config` `$EDITOR` hand-off (ARCH §2.2, §3.4): open the
/// authoring checkout so the user edits the control files, treating a
/// non-zero editor exit as a failed edit. `$EDITOR` may carry arguments,
/// so it runs through `sh -c`.
pub(crate) fn edit_in_editor(dir: &Path) -> io::Result<()> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("exec {editor} \"$1\""))
        .arg("sh")
        .arg(dir)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("editor exited with {status}")))
    }
}
