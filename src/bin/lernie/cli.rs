//! Interactive / exec CLI slivers of the `lernie` binary that cannot be
//! unit-tested and so live at the coverage-exempt bin seam (ARCH §3.4):
//! the `lernie config` `$EDITOR` hand-off and the `lernie advance` exec
//! baton. Everything testable lives in the library; these are the thin
//! process-spawning edges.

use lernie::prompt;
use std::io;
use std::path::Path;
use std::process::ExitCode;

use super::fail;

/// The `lernie config` `$EDITOR` hand-off (ARCH §2.2, §3.4): open the
/// authoring checkout so the user edits the control files, treating a
/// non-zero editor exit as a failed edit. The verb's one untestable
/// sliver — origin resolution and the rest live in
/// [`lernie::template::authoring::from_cli`]; only the interactive spawn
/// is here. `$EDITOR` may carry arguments, so it runs through `sh -c`.
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

/// CLI handler for `lernie advance <workspace> <agent>` (ARCH §6): one
/// hop of the workflow chain. The library does everything up to the
/// exec ([`prompt::dispatch::advance::cli::cli_run`]); this shim only
/// performs the `exec` itself — a successful `execve` replaces this
/// image (the §6 exec baton, lock fd riding it), so the call returning
/// at all is the failure path.
pub(crate) fn run_advance_cli(workspace: &Path, agent: &str) -> ExitCode {
    prompt::stop::become_pgid_leader(); // §2.9: every driver takes its own pgid
    prompt::install_stop_handler(); // §2.9 step-3 stopped deposit
    match prompt::dispatch::advance::cli::cli_run(workspace, agent) {
        Ok(prompt::dispatch::advance::cli::AdvanceHandoff::Exec(mut cmd)) => {
            use std::os::unix::process::CommandExt;
            fail("lernie advance: exec successor", cmd.exec())
        }
        Ok(prompt::dispatch::advance::cli::AdvanceHandoff::Done(_)) => ExitCode::SUCCESS,
        Err(e) => fail("lernie advance", e),
    }
}
