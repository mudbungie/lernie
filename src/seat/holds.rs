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
    let mut rows = vec![row(OWN, &own(data_root).unwrap_or_else(|why| why))];
    for held in entries::read_dir(&entries::dir(data_root)) {
        let label = label(&held.leaf, &held.workspace);
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
/// the window can paint its channels before any engine is up.
///
/// Each carries the name the workspace bears on its host, which is the fact
/// that decides what a gesture aimed at one of its rows must be addressed as
/// ([`crate::ui::Channel::address`]). The flat root carries `None`, because it
/// rewrites nothing.
///
/// **And each carries why it has no walls yet** ([`crate::ui::Held`],
/// bl-08b6): a channel this box cannot dial arrives already carrying the
/// sentence [`listing`] prints for it, so the window's first run says what it
/// is waiting for instead of painting a section header over a blank.
pub fn channels(data_root: &Path) -> Vec<crate::ui::Chunk> {
    let mut held = vec![chunk(
        crate::ui::Channel {
            name: OWN.to_owned(),
            named_there: None,
        },
        own(data_root).err(),
    )];
    held.extend(
        entries::read_dir(&entries::dir(data_root))
            .into_iter()
            .map(|entry| {
                chunk(
                    crate::ui::Channel {
                        name: entry.leaf,
                        named_there: Some(entry.workspace),
                    },
                    entry.channel.err(),
                )
            }),
    );
    held
}

/// One seated section: the channel, and the reason it has no walls where this
/// box already knows one.
fn chunk(channel: crate::ui::Channel, unheld: Option<String>) -> crate::ui::Chunk {
    crate::ui::Chunk {
        channel,
        held: unheld.map_or(crate::ui::Held::Unheard, crate::ui::Held::Unheld),
        ..crate::ui::Chunk::default()
    }
}

/// **The name this box knows a channel by**, as every surface spells it: the
/// leaf, plus the name its workspace bears on its host where the two differ.
///
/// One home, so the listing and the fan ([`super::fan`]) cannot spell one
/// channel two ways — a stamp that disagreed with the roster would be worse
/// than no stamp.
pub(super) fn label(leaf: &str, workspace: &str) -> String {
    if leaf == workspace {
        leaf.to_owned()
    } else {
        format!("{leaf} (named {workspace:?} on its host)")
    }
}

/// **The channels this box holds, named in one clause** — the enumeration a
/// refusal about an unresolved name owes the operator.
///
/// It is [`listing`]'s subject without [`listing`]'s state: a refusal that
/// says *"there is no channel called that"* is only half an answer, and the
/// other half is the set the operator meant to pick from. Read off the same
/// disk, so it can never name a channel the roster does not.
pub(super) fn names(data_root: &Path) -> String {
    std::iter::once(OWN.to_owned())
        .chain(
            entries::read_dir(&entries::dir(data_root))
                .into_iter()
                .map(|held| format!("{:?}", held.leaf)),
        )
        .collect::<Vec<String>>()
        .join(", ")
}

/// **The rename remedy**, which is the one an unresolved name almost always
/// wants: material that is present, valid and correctly permissioned but filed
/// under a directory name no gesture routes to is renamed here, with `mv`. The
/// mint ([`material::REMEDY`]) is the remedy for material that does not exist,
/// and offering it for material that does sends the operator back to the box
/// holding the CA to issue a second leaf that lands in the same wrong
/// directory.
pub(super) fn rename(data_root: &Path, name: &str) -> String {
    format!(
        "if {name:?}'s material is already here under another directory name, \
         rename that directory (`mv` it under {}) rather than minting a second \
         leaf",
        entries::dir(data_root).display()
    )
}

/// The flat root's state: its address, or the sentence saying why it has none.
///
/// **One read, two readers.** [`listing`] prints whichever it is; [`channels`]
/// takes only the `Err`, because an address is what a channel answers *with*
/// and a sentence is what it answers *instead*.
fn own(data_root: &Path) -> Result<String, String> {
    let dir = entries::flat(data_root);
    match material::read_dir(&dir) {
        Ok(Some(held)) => Ok(held.address),
        Ok(None) => Err(format!(
            "nothing provisioned at {}: {}",
            dir.display(),
            material::REMEDY
        )),
        Err(refusal) => Err(refusal),
    }
}

/// One listing row: the name this box knows a channel by, and what it is.
fn row(label: &str, state: &str) -> String {
    format!("{label}\n    {state}")
}
