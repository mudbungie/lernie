//! **What this box holds** — every channel it can reach, said without dialling
//! any of them.
//!
//! Split from [`super`] at the design-time budget on the seam the suite is
//! already split along: [`super`] is which engine a gesture reaches and what it
//! carries there, and this is what the box says it has before any gesture is
//! typed. Both read the same entries; only one of them opens a socket.

use std::path::Path;

use super::flat;
use crate::channel::{Channel, entries, material};
use crate::cli::Verdict;

/// What this box calls its own engine — the flat root's label, which is not a
/// name an operator chose and so is not one a gesture may carry.
pub const OWN: &str = "(this box's own engine)";

/// **What this box holds** — every channel it can reach, said without dialling
/// any of them.
///
/// A listing that honestly reports nothing is a successful listing, so a box
/// holding no channel at all answers zero rather than refusing: it is a fact
/// about provisioning, and the operator asking is the operator who would fix
/// it. Each entry carries its own state for §8.2's reason — a half-provisioned
/// entry says so beside its neighbours rather than costing them the listing.
pub fn listing(data_root: &Path) -> Verdict {
    let mut rows = vec![row(OWN, &own(data_root))];
    for held in entries::read_dir(&entries::dir(data_root)) {
        let label = if held.workspace == held.leaf {
            held.leaf.clone()
        } else {
            format!("{} (named {:?} on its host)", held.leaf, held.workspace)
        };
        let state = held
            .channel
            .map_or_else(|state| state, |material| material.address);
        rows.push(row(&label, &state));
    }
    Verdict::ok(rows.join("\n"))
}

/// **Open the channel this box knows by `name`** — the flat root's label, or
/// an entry's leaf.
///
/// It addresses a **channel**, where [`ask`] addresses a **gesture**, and the
/// distinction is real rather than a convenience: the roster read names no
/// workspace at all, so there is nothing for §8.2's mapping to resolve and the
/// channel has to be named outright. Every other question carries an address
/// and goes through [`route`], where the mapping is spent exactly once.
pub fn dial(data_root: &Path, name: &str) -> Result<Channel, String> {
    if name == OWN {
        return flat(data_root);
    }
    entries::read_dir(&entries::dir(data_root))
        .into_iter()
        .find(|held| held.leaf == name)
        .ok_or_else(|| format!("this box holds no channel named {name:?}"))?
        .open()
}

/// **Every channel this box holds, as the window stamps its rows** (§8.2).
///
/// The same enumeration [`listing`] prints, typed instead of rendered — this
/// box's own engine first, then one per entry in leaf order. It reads the disk
/// and **dials nothing**: what a box holds is a fact about its own files, so
/// the window can paint its channels before any engine is up and can say
/// plainly that it holds none.
///
/// Each carries the name the workspace bears on its host, which is the fact
/// that decides what a gesture aimed at one of its rows must be addressed as
/// ([`crate::ui::Channel::address`]). The flat root carries `None`, because it
/// rewrites nothing.
pub fn channels(data_root: &Path) -> Vec<crate::ui::Channel> {
    let mut held = vec![crate::ui::Channel {
        name: OWN.to_owned(),
        named_there: None,
    }];
    held.extend(
        entries::read_dir(&entries::dir(data_root))
            .into_iter()
            .map(|entry| crate::ui::Channel {
                name: entry.leaf,
                named_there: Some(entry.workspace),
            }),
    );
    held
}

/// The flat root's state, said the way an entry's is.
fn own(data_root: &Path) -> String {
    let dir = entries::flat(data_root);
    match material::read_dir(&dir) {
        Ok(Some(held)) => held.address,
        Ok(None) => format!("nothing provisioned at {}", dir.display()),
        Err(refusal) => refusal,
    }
}

/// One listing row: the name this box knows a channel by, and what it is.
fn row(label: &str, state: &str) -> String {
    format!("{label}\n    {state}")
}
