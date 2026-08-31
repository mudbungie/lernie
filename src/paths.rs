//! **Where this box keeps what the operator gave it.**
//!
//! One directory holds the seat's durable facts, and today they are all channel
//! material: this box's own client relationship (`wire/`) and one entry per
//! workspace held elsewhere (`wire/workspaces/<leaf>/`). All of it is
//! **operator-provisioned and irreplaceable by anything the seat can do**,
//! which is why nothing the seat generates may ever be written beside it — a
//! regenerable subtree under the same root would make a rebuild a revocation.
//!
//! **So there are two roots and the split is that hazard** (bl-0fba). REMOTE §7
//! rules that per-seat UI state — focus, scroll, tab selection, drafts — never
//! crosses the boundary and is the seat's own, so the window is durable state
//! this box GENERATES rather than state an operator carried here. It lives
//! under [`state_root`] and never under [`data_root`], and the rule that
//! separates them is not tidiness:
//!
//! - **What is under the data root cannot be rebuilt by anything on this box.**
//!   Deleting it is a revocation and re-minting is an act on another machine by
//!   another hand (REMOTE §1.4).
//! - **What is under the state root can be deleted at any time with no cost but
//!   a forgotten selection**, which is what makes it safe to write, safe to
//!   rewrite on every close, and safe to advise an operator to remove.
//!
//! A regenerable subtree beside irreplaceable material makes the second look
//! like the first: an operator clearing the seat's saved place would be one
//! `rm` away from clearing the certificate that gets them back in. XDG already
//! draws exactly this line — `XDG_DATA_HOME` for what must survive,
//! `XDG_STATE_HOME` for what a program remembers between runs — so the split
//! costs one variable and no invention.
//!
//! **Two variables per root, and no knob of the seat's own.** The XDG convention names
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
/// The XDG variable that names the state root outright.
const XDG_STATE: &str = "XDG_STATE_HOME";
/// The variable both conventions' defaults are derived from.
const HOME_VAR: &str = "HOME";
/// The data convention's default, relative to [`HOME_VAR`].
const DEFAULT: &str = ".local/share";
/// The state convention's default, relative to [`HOME_VAR`].
const DEFAULT_STATE: &str = ".local/state";

/// This box's data root, read from this process's own environment.
pub fn data_root() -> Result<PathBuf, String> {
    root_of(std::env::var_os(XDG), std::env::var_os(HOME_VAR))
}

/// **This box's state root** — where the seat keeps what it generates about
/// itself, and never where the operator's material lives (see the module doc).
pub fn state_root() -> Result<PathBuf, String> {
    keep_of(std::env::var_os(XDG_STATE), std::env::var_os(HOME_VAR))
}

/// [`data_root`]'s pure core — the environment as two values, so the rule can
/// be read and tested without a process to fold one into.
///
/// An empty variable is an unset one: a launcher that exports a name with no
/// value has said nothing, and treating it as a root would name the filesystem
/// root's own `lernie` directory.
fn root_of(xdg: Option<OsString>, home: Option<OsString>) -> Result<PathBuf, String> {
    under(xdg, home, DEFAULT).ok_or_else(|| {
        format!(
            "this box's data root is not named: set {XDG}, or {HOME_VAR} so that \
             {DEFAULT}/{HOME} under it can be found. lernie will not guess — an \
             operator's certificates would land wherever this process happened to \
             be started."
        )
    })
}

/// [`state_root`]'s pure core. **The same ladder and a different sentence**: the
/// two roots differ in one variable and one default, so the rule is written
/// once, and what a reader needs is not the ladder repeated but what is lost
/// when this particular root cannot be named.
fn keep_of(xdg: Option<OsString>, home: Option<OsString>) -> Result<PathBuf, String> {
    under(xdg, home, DEFAULT_STATE).ok_or_else(|| {
        format!(
            "this box's state root is not named: set {XDG_STATE}, or {HOME_VAR} \
             so that {DEFAULT_STATE}/{HOME} under it can be found. Nothing is \
             lost but what the window remembers between runs."
        )
    })
}

/// The convention's ladder: the variable that names a root outright, else the
/// home-relative default under it.
fn under(xdg: Option<OsString>, home: Option<OsString>, default: &str) -> Option<PathBuf> {
    if let Some(root) = stated(xdg) {
        return Some(root.join(HOME));
    }
    Some(stated(home)?.join(default).join(HOME))
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
