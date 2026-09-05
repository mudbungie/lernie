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
//! [`with_workspace`](crate::envelope::with_workspace), so a gesture cannot
//! cross renamed down one path and unrenamed down another. Where the two names
//! agree — the ordinary provisioning — the operator's own envelope crosses byte
//! for byte. What it chose comes back out beside the channel ([`Routed`]), so
//! nothing downstream has to guess at it a second time.

use std::path::Path;

use serde_json::Value;

/// The §8.4 enrollment act, whose reply is a picture rather than a stream.
mod enroll;
/// A gesture that names no workspace, asked of every channel this box holds.
mod fan;
/// What this box says it holds, said without dialling any of it.
mod holds;
/// Which channel a gesture goes down, and what it carries there.
mod route;
/// The §8.1 start family's two acts, spelled as one word.
mod start;

pub use enroll::enroll;
pub use fan::fanned;
pub use holds::{OWN, channels, dial, listing};
pub use route::{Routed, route};
pub use start::start;

use crate::channel::Reach;
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
    match sent(data_root, envelope) {
        Ok(stream) => Verdict::answered(lines(&stream), envelope::succeeded(&stream)),
        Err(reach) => Verdict::failed(reach.said()),
    }
}

/// **One gesture, spent**: routed and asked, as one act.
///
/// Every caller does both and neither half is useful alone — a channel opened
/// and not asked is a connection nobody wanted — so the pair is one function
/// and the two failures collapse into the one sentence they always were.
///
/// **They collapse into one SENTENCE and not into one outcome** (bl-3969). A
/// gesture this box could not route never crossed, so it joins everything
/// [`crate::channel::Channel::ask`] classes [`Reach::Unsent`]; what the
/// [`Reach`] carries past this point is the fact a caller with an ACT in hand
/// needs and a caller printing a sentence does not.
pub(crate) fn sent(data_root: &Path, envelope: &Value) -> Result<Vec<Value>, Reach> {
    let (channel, carried) = route(data_root, envelope).sent.map_err(Reach::Unsent)?;
    channel.ask(&carried)
}

/// **The reply stream as this seat's product**: one envelope per line, and the
/// one place that shape is written — [`start`] prints two streams the same way,
/// and two spellings of "what a seat printed" is two products.
pub(crate) fn lines(stream: &[Value]) -> String {
    stream
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join("\n")
}

#[cfg(test)]
mod tests;
