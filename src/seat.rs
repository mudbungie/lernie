//! **The seat**: which engine a gesture reaches, and what it carries there
//! (yog's `docs/REMOTE.md` §8.2; DESIGN §4.7).
//!
//! REMOTE §8.2 fixes the rule and this module is the whole of it: *"the
//! gesture's workspace name is resolved over the entries this box holds first;
//! a name no entry holds — and a gesture naming no workspace — goes where it
//! always went, the flat directory's client material."* So the flat root stays
//! what it has always been — the box's own client relationship, held without
//! naming it — and everything beyond the box's own engine is an entry.
//!
//! **An entry that exists is the answer to its name even when it cannot be
//! dialled.** A half-provisioned entry refuses with its own sentence rather
//! than falling through to the flat root, which would send a gesture to the
//! wrong engine on the strength of a missing file.
//!
//! **The leaf↔host-name mapping is spent at exactly one place, and this is
//! it.** [`route`] is the only function in this crate that calls
//! [`envelope::with_workspace`], so a gesture cannot cross renamed down one
//! path and unrenamed down another. Where the two names agree — the ordinary
//! provisioning — the operator's own envelope crosses byte for byte.

use std::path::Path;

use serde_json::Value;

use crate::channel::material::{self, REMEDY};
use crate::channel::{Channel, entries};
use crate::cli::Verdict;
use crate::envelope;

/// Send one gesture envelope down the channel its workspace names, and answer
/// with the engine's reply stream.
///
/// **It takes the envelope, never the text.** Whether a body is a gesture at
/// all is decided by what the caller typed and by nothing about this box, so it
/// is settled in [`crate::cli`] — where the refusal is a value a test reads
/// back and where it costs no connection — and a typed verb and a hand-written
/// `ask` arrive here as the same value.
///
/// Two failures remain and they are two different things: a channel that will
/// not open or will not answer is a fact about this box or the far end, and
/// earns the sentence alone; a reply that says `ok: false` is the engine
/// **answering**, so it goes to stdout with the rest of the stream and only the
/// exit code says no.
pub fn ask(data_root: &Path, envelope: &Value) -> Verdict {
    let (channel, carried) = match route(data_root, envelope) {
        Ok(routed) => routed,
        Err(refusal) => return Verdict::failed(refusal),
    };
    match channel.ask(&carried) {
        Ok(stream) => answer(&stream),
        Err(refusal) => Verdict::failed(refusal),
    }
}

/// **Which channel this gesture goes down, and what it carries there** (§8.2).
///
/// The envelope's workspace name is resolved over this box's entries *first*; a
/// name no entry holds — and a gesture naming no workspace — goes to the flat
/// root. The re-encode happens here and only here, and only where an entry
/// renames: an entry's leaf is the client's name for the workspace and its
/// `workspace` file is what that workspace answers to on its host.
pub fn route(data_root: &Path, envelope: &Value) -> Result<(Channel, Value), String> {
    let named = envelope::workspace(envelope).and_then(|name| {
        entries::read_dir(&entries::dir(data_root))
            .into_iter()
            .find(|held| held.leaf == name)
    });
    let Some(entry) = named else {
        return Ok((flat(data_root)?, envelope.clone()));
    };
    let carried = if entry.workspace == entry.leaf {
        envelope.clone()
    } else {
        envelope::with_workspace(envelope, &entry.workspace)
    };
    Ok((entry.open()?, carried))
}

/// This box's own channel — the flat root's, which is where every gesture with
/// no entry to resolve goes.
///
/// Absent material is a refusal here rather than the silence it is at the
/// entries directory: a gesture that resolved to nothing has nowhere to go, and
/// the remedy is the same out-of-channel act (REMOTE §1.4).
fn flat(data_root: &Path) -> Result<Channel, String> {
    let dir = entries::flat(data_root);
    match material::read_dir(&dir)? {
        // A `:0` is a self-provisioning engine's request for a kernel-chosen
        // port (REMOTE §8): only the engine that bound it knows what it became,
        // and it tells its own in-process window in RAM. So there is nothing
        // here to dial, and saying so beats the raw connect error port zero
        // earns.
        Some(held) if held.address.ends_with(":0") => Err(format!(
            "{} names {} — a kernel-chosen port only that engine's own window is \
             told; a seat wants a stated address",
            dir.join(material::ADDRESS).display(),
            held.address
        )),
        Some(held) => Channel::open(&held),
        None => Err(format!(
            "no wire provisioned at {}: {REMEDY}",
            dir.display()
        )),
    }
}

/// The reply stream as this seat's product: one envelope per line, and the exit
/// code read off the last frame's verdict.
fn answer(stream: &[Value]) -> Verdict {
    let text = stream
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    Verdict::answered(text, envelope::succeeded(stream))
}

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

#[cfg(test)]
mod tests;
