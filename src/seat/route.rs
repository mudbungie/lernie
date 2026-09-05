//! **Which channel a gesture goes down, and what it carries there** (yog's
//! `docs/REMOTE.md` §8.2; DESIGN §4.7).
//!
//! Split from [`super`] at the design-time budget on the seam the module
//! already had: [`super`] is one gesture *spent* — routed, asked, and answered
//! as a product — and this is the resolution alone, which opens a channel and
//! sends nothing.
//!
//! **The leaf↔host-name mapping is spent here and nowhere else.** [`route`] is
//! the only function in this crate that calls
//! [`with_workspace`](crate::envelope::with_workspace), so a gesture cannot
//! cross renamed down one path and unrenamed down another — and the seat-side
//! name of the channel it chose comes out of [`Routed`] for the same reason,
//! rather than being guessed at again by whoever files the answer (bl-c70d).

use std::path::Path;

use serde_json::Value;

use super::{OWN, holds};
use crate::channel::material::{self, REMEDY};
use crate::channel::{Channel, entries};
use crate::envelope;

/// **One routed gesture**: where it goes, what it carries there, and what this
/// box calls the channel it went down.
///
/// The seat-side name is ANSWERED rather than discarded, because the caller
/// filing a receipt has nowhere honest to get it (bl-c70d): a frame's aim is
/// where a gesture was *composed*, and an operator may compose one aimed at a
/// wall on one channel while a control fires at a row on another. §8.2's
/// mapping stays spent at [`route`] and only there — the name comes out of it
/// and is never re-derived downstream.
///
/// It is answered whether or not anything opened, for the same reason: a leg
/// that never crossed still has a sentence to paint, and the section it belongs
/// under is the one it would have crossed on.
pub struct Routed {
    /// **What this box calls the channel** — [`OWN`] for the flat root, the
    /// entry's leaf for an entry, and the name that resolved to neither where
    /// a selector named no channel this box holds.
    ///
    /// It is [`crate::ui::Channel`] and not a bare name because the window
    /// stamps a whole channel on the rows that came down one; `dials` is what
    /// was actually dialled, so a channel that would not open carries `None`
    /// rather than a claim about an address nothing reached.
    pub down: crate::ui::Channel,
    /// **The opened channel and the envelope as it crosses it**, or the
    /// sentence saying why the gesture goes nowhere.
    pub sent: Result<(Channel, Value), String>,
}

/// **Which channel this gesture goes down, and what it carries there** (§8.2).
///
/// The envelope's workspace name is resolved over this box's entries *first*; a
/// name no entry holds — and a gesture naming no workspace — goes to the flat
/// root. The re-encode happens here and only here, and only where an entry
/// renames: an entry's leaf is the client's name for the workspace and its
/// `workspace` file is what that workspace answers to on its host.
pub fn route(data_root: &Path, envelope: &Value) -> Routed {
    let asked = envelope::workspace(envelope);
    let named = asked.as_ref().and_then(|name| {
        entries::read_dir(&entries::dir(data_root))
            .into_iter()
            .find(|held| &held.leaf == name)
    });
    match (named, asked) {
        (Some(entry), _) => seated(entry, envelope),
        (None, None) => own(data_root, envelope),
        (None, Some(name)) => unresolved(data_root, envelope, &name),
    }
}

/// **The gesture an entry answers**: rewritten to the host's spelling where the
/// entry renames, and stamped with the leaf this box knows it by.
fn seated(entry: crate::channel::entries::Entry, envelope: &Value) -> Routed {
    let carried = if entry.workspace == entry.leaf {
        envelope.clone()
    } else {
        envelope::with_workspace(envelope, &entry.workspace)
    };
    let opened = entry.open();
    Routed {
        down: stamp(entry.leaf, Some(entry.workspace), opened.as_ref().ok()),
        sent: opened.map(|channel| (channel, carried)),
    }
}

/// **The gesture this box's own engine answers** — the flat root's, which is
/// where every gesture with no entry to resolve goes.
fn own(data_root: &Path, envelope: &Value) -> Routed {
    let opened = flat(data_root);
    Routed {
        down: stamp(OWN.to_owned(), None, opened.as_ref().ok()),
        sent: opened.map(|channel| (channel, envelope.clone())),
    }
}

/// **What this box calls a channel, as the window stamps the rows that came
/// down one** ([`crate::ui::Channel`]) — taken from the channel that opened,
/// so `dials` is an address something reached rather than one a file names.
fn stamp(
    name: String,
    named_there: Option<String>,
    opened: Option<&Channel>,
) -> crate::ui::Channel {
    crate::ui::Channel {
        name,
        named_there,
        dials: opened.map(Channel::address),
    }
}

/// **Where a NAMED workspace goes when no entry holds it**, and what the
/// refusal is about when it goes nowhere.
///
/// §8.2's fallthrough stands — *"a name no entry holds … goes where it always
/// went, the flat directory's client material"* — because the flat engine's own
/// workspaces are named and held nowhere else, and a seat cannot know that
/// namespace without asking. Two things around it are this seat's, and both
/// were wrong (bl-d574):
///
/// - **A gesture whose op takes no workspace is naming a CHANNEL and nothing
///   else**, since there is no parameter for the far end to read: `lernie ask
///   '{"op":"workspaces","workspace":"<leaf>"}'` is how an operator asks one
///   entry for its roster. So a name no entry holds has no downstream reader to
///   refuse it, and falling through answers `ok` from a channel nobody named.
///   It refuses here instead, naming what it looked for. Which ops those are is
///   read off [`crate::verbs`]'s one table rather than listed again.
/// - **The fallthrough's refusal was about the wrong subject.** Where the flat
///   root holds nothing, the sentence said "no wire provisioned at `wire/`" —
///   true, and about a directory the operator never asked about, with a remedy
///   (mint a second leaf) that is destructive of their time. It now says which
///   name failed to resolve, which channels this box holds, and that the likely
///   remedy is a rename.
fn unresolved(data_root: &Path, envelope: &Value, name: &str) -> Routed {
    let selector = envelope
        .get(envelope::OP)
        .and_then(Value::as_str)
        .and_then(crate::verbs::find)
        .is_some_and(|verb| !verb.addresses_a_workspace());
    if selector {
        return Routed {
            down: stamp(name.to_owned(), None, None),
            sent: Err(format!(
                "this box holds no channel named {name:?}: that op takes no \
                 workspace, so the name can only name a channel to ask. It holds \
                 {}. {}",
                holds::names(data_root),
                holds::rename(data_root, name)
            )),
        };
    }
    let fell = own(data_root, envelope);
    Routed {
        down: fell.down,
        sent: fell.sent.map_err(|refusal| {
            format!(
                "no entry here holds {name:?}, so the gesture went to this box's \
                 own engine (REMOTE §8.2), which answers: {refusal}. This box holds \
                 {}. {}",
                holds::names(data_root),
                holds::rename(data_root, name)
            )
        }),
    }
}

/// This box's own channel — the flat root's, which is where every gesture with
/// no entry to resolve goes.
///
/// Absent material is a refusal here rather than the silence it is at the
/// entries directory: a gesture that resolved to nothing has nowhere to go, and
/// the remedy is the same out-of-channel act (REMOTE §1.4).
pub(super) fn flat(data_root: &Path) -> Result<Channel, String> {
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
