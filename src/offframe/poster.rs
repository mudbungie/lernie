//! **The poster**: one pass over what a frame composed.
//!
//! It is a thread of its own and not a leg of the asker's pass, because an act
//! must not wait behind a read that is mid-flight. An operator who has pressed
//! send has already decided; making them wait for a roster refresh to finish
//! first is the seat inserting itself into their intent.
//!
//! **A receipt is a frame like any other**, so it goes back through the same
//! door every answer does and lands as content or as the notice bar. Nothing
//! here reads what an act earned.
//!
//! # A gesture naming no workspace is FANNED, not routed (bl-40ec)
//!
//! `crate::seat::route` resolves an envelope's workspace over this box's
//! entries and falls through to the flat root when there is none — which is
//! right for `lernie ask`, where the operator named one channel by hand, and
//! wrong for a frame. A window that composed `workspaces` would have asked
//! this box's own engine and said nothing about the rest, which is bl-0d54's
//! defect one surface over.
//!
//! So the poster asks the envelope instead: a gesture with no workspace field
//! has no way to name a channel, so its subject is every channel the standing
//! set holds ([`crate::verbs::Verb::addresses_a_workspace`] is the same rule
//! on the table, and `crate::cli::Decided::Fanned` and `crate::seat::fan` are
//! it on argv). That is what lets a frame reach the three window-level reads
//! at all — the roster refresh, the engines' own verb table and a search —
//! and it is why each answer is stamped with the channel it came down rather
//! than with the aim.

use std::path::Path;

use serde_json::Value;

use crate::state::{Link, Said};
use crate::ui::Channel;

/// Send everything the frame composed since the last pass.
///
/// The receipt of a **routed** gesture is stamped with the aimed channel,
/// which is where a composed gesture came from. A gesture composed with no aim
/// is still sent — the address it carries is what routes it, and the address
/// is the whole of what routing needs — and its receipt is stamped with a
/// channel that names nothing, because a stamp this seat cannot honestly make
/// is not one it should invent. A **fanned** gesture needs no such guess: each
/// leg knows the channel it opened.
pub fn tick(link: &Link, root: &Path) {
    let standing = link.standing();
    let channel = standing
        .aimed()
        .map(|(channel, _)| channel)
        .unwrap_or_default();
    for envelope in link.compose() {
        if crate::envelope::workspace(&envelope).is_none() {
            for held in &standing.channels {
                super::down(link, root, held, &envelope);
            }
            continue;
        }
        routed(link, root, &channel, &envelope);
    }
}

/// One gesture down the channel its workspace names, and the receipt filed
/// against the aim.
fn routed(link: &Link, root: &Path, channel: &Channel, envelope: &Value) {
    match crate::seat::route(root, envelope).and_then(|(open, carried)| open.ask(&carried)) {
        Ok(stream) => super::file(link, channel, stream),
        Err(why) => link.heard(channel, Said::Unreachable(why)),
    }
}

#[cfg(test)]
mod tests;
