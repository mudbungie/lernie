//! **Where the seat was pointed**, remembered between runs (yog's
//! `docs/REMOTE.md` §7; DESIGN §4.13).
//!
//! REMOTE §7 rules that per-seat UI state never crosses the boundary and is the
//! seat's own. This is the first of it that is durable, and where it goes is
//! the decision that mattered: under `paths::state_root` and never under
//! `paths::data_root`, because everything under the second is
//! operator-provisioned and irreplaceable by anything on this box, and a
//! regenerable subtree beside it would make a rebuild look like a revocation.
//! The whole of that argument is `crate::paths`'s module doc.
//!
//! # It may never become a way for the seat to fail to start
//!
//! [`read`] has exactly one answer for a file that is absent, unreadable,
//! truncated, or written by a build that spelled things differently: **no
//! place**, which is a window that opens on the roster — the same window a
//! first run gets. There is no refusal here and no repair path, because a
//! forgotten selection is a keypress and a startup error is an outage.
//!
//! That is also why the aim is not checked against anything. A wall that has
//! gone is a wall no channel resolves, and the standing set already declines to
//! ask about one (`crate::state::Standing::aimed`) — so a stale place is inert
//! by a rule that was already there, and validating it here would be a second
//! answer to a question already settled.
//!
//! # What it holds, and how it grows
//!
//! One fact today: the wall the window was aimed at. **A JSON object rather
//! than two lines**, so the next fact REMOTE §7 names — a scroll, a selection,
//! a draft — is a key beside this one rather than a format. An unknown key is
//! ignored and a missing key is absence, which is the reply vocabulary's own
//! rungs 3 and 4 applied to this box's own file: a build that reads a file a
//! newer build wrote loses what it does not know and keeps what it does.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::ui::Aim;

/// The file, under the state root. One name, so there is nothing to configure.
const FILE: &str = "place.json";
/// The one key today, and the two the aim is written as.
const AIM: &str = "aim";
const CHANNEL: &str = "channel";
const ADDRESS: &str = "address";

/// Where the place is kept under `root`.
pub fn at(root: &Path) -> PathBuf {
    root.join(FILE)
}

/// **The wall the window was last aimed at**, or nothing — which is every way
/// this can fail and the answer a first run gets.
pub fn read(root: &Path) -> Option<Aim> {
    let held: Value = serde_json::from_str(&std::fs::read_to_string(at(root)).ok()?).ok()?;
    let text = |key| held.get(AIM)?.get(key)?.as_str().map(str::to_owned);
    Some(Aim {
        channel: text(CHANNEL)?,
        address: text(ADDRESS)?,
    })
}

/// **Write the place down.** Aimed at nothing is a place too — an operator who
/// left the roster comes back to it — so this is called with whatever the last
/// frame held rather than only when there is something to say.
///
/// It answers a refusal rather than swallowing one. A read that fails has a
/// correct answer and this does not: the only alternative to saying so is
/// losing the operator's place in silence, and by the time this runs there is
/// no window left to paint it in.
pub fn write(root: &Path, aim: Option<Aim>) -> Result<(), String> {
    let body = json!({
        AIM: aim.map(|aim| json!({ CHANNEL: aim.channel, ADDRESS: aim.address })),
    });
    std::fs::create_dir_all(root).map_err(|e| format!("{}: {e}", root.display()))?;
    std::fs::write(at(root), body.to_string()).map_err(|e| format!("{}: {e}", at(root).display()))
}

#[cfg(test)]
mod tests;
