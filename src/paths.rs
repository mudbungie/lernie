//! **Where this box keeps what the operator gave it.**
//!
//! One directory holds the seat's durable facts, and today they are all channel
//! material: this box's own client relationship (`wire/`) and one entry per
//! workspace held elsewhere (`wire/workspaces/<leaf>/`). All of it is
//! **operator-provisioned and irreplaceable by anything the seat can do**,
//! which is why nothing the seat generates may ever be written beside it — a
//! regenerable subtree under the same root would make a rebuild a revocation.
//! The seat generates nothing today, and this is the note that says why it must
//! not start here. (Per-seat UI state, when the window lands, is REMOTE §7's
//! and is durable in a different sense; it gets its own subtree or it gets the
//! same hazard.)
//!
//! **Two variables, and no knob of the seat's own.** The XDG convention names
//! the directory, so a box that already places application data somewhere
//! places lernie's there too, and there is nothing to configure and nothing
//! that can disagree with it. A third variable — a `LERNIE_HOME` — would be a
//! second authority for one fact, and the one thing it would buy (a scratch
//! root for tests) is already had by every function below the process edge
//! taking the root as an argument.
//!
//! **`LERNIE_HOME` is doubly refused, and the second reason is the fence.**
//! That variable *existed*, and it named the agent-loop engine's harness root
//! through `lernie` 0.0.x. It is `LITANY_HOME` now (REMOTE §12), and reviving
//! the old spelling for a different program would make one name mean two things
//! on a box that has both installed.
//!
//! **The data root has the same collision and it is benign, deliberately.** A
//! box that ran the pre-fence engine may hold a `$XDG_DATA_HOME/lernie` from
//! that era. This crate reads exactly two paths under its root — `wire/` and
//! `wire/workspaces/` — and the engine's home held none of that shape, so the
//! two coexist without a file in common. Picking a different directory to avoid
//! a directory name would have been the seat conceding the name it was given.
//!
//! **Neither variable set is a refusal, not a guess.** A relative fallback
//! would put an operator's certificates wherever the launcher happened to start
//! the process, which is a place nobody chose and nobody can find again.

use std::ffi::OsString;
use std::path::PathBuf;

/// The directory the seat's own data lives in, under whichever root names it.
const HOME: &str = "lernie";
/// The XDG variable that names the data root outright.
const XDG: &str = "XDG_DATA_HOME";
/// The variable the convention's default is derived from.
const HOME_VAR: &str = "HOME";
/// The convention's default, relative to [`HOME_VAR`].
const DEFAULT: &str = ".local/share";

/// This box's data root, read from this process's own environment.
pub fn data_root() -> Result<PathBuf, String> {
    root_of(std::env::var_os(XDG), std::env::var_os(HOME_VAR))
}

/// [`data_root`]'s pure core — the environment as two values, so the rule can
/// be read and tested without a process to fold one into.
///
/// An empty variable is an unset one: a launcher that exports a name with no
/// value has said nothing, and treating it as a root would name the filesystem
/// root's own `lernie` directory.
fn root_of(xdg: Option<OsString>, home: Option<OsString>) -> Result<PathBuf, String> {
    if let Some(root) = stated(xdg) {
        return Ok(root.join(HOME));
    }
    if let Some(root) = stated(home) {
        return Ok(root.join(DEFAULT).join(HOME));
    }
    Err(format!(
        "this box's data root is not named: set {XDG}, or {HOME_VAR} so that \
         {DEFAULT}/{HOME} under it can be found. lernie will not guess — an \
         operator's certificates would land wherever this process happened to \
         be started."
    ))
}

/// One variable's value, or nothing when it said nothing.
fn stated(value: Option<OsString>) -> Option<PathBuf> {
    let held = value?;
    if held.is_empty() {
        return None;
    }
    Some(PathBuf::from(held))
}

#[cfg(test)]
mod tests;
