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
//! # An act is sent exactly once, and a lost reply is never a resend
//!
//! yog's `docs/REMOTE.md` §3: *"A lost reply leaves an act IN DOUBT, and the
//! recovery is a read — never a resend."* This pass is where a seat would have
//! to break that, and it does not: [`crate::state::Link::compose`] is a take,
//! the drained envelopes live in this function's own loop, and no arm below
//! re-queues one. There is no retry, no backoff and no reconnect-and-replay
//! anywhere in this crate — the only repetition it owns is
//! [`crate::offframe::pump`]'s cadence, which re-derives the standing READS
//! from the model and never touches this queue.
//!
//! **What the failure arm owes is the paint** ([`crate::channel::Reach`]). An
//! act that crossed and was not answered may have run, so it is said as IN
//! DOUBT with the contract's own remedy; one that never left this box did not
//! happen, so it is said as not sent and doing it again is safe. Both are facts
//! about an **exchange**, so both go to the shell's bar — where a refusal goes
//! — and never to a channel's roster section, which is a relationship (REMOTE
//! §8.2, bl-e620). That placement is load-bearing rather than tidy: the section
//! is the slot the roster read owns, and the asker answers `workspaces` on
//! every beat, so an act's sentence written there is erased within 750 ms by a
//! read that succeeded.
//!
//! **A posted READ keeps the read arm**, because §4.21's three window-level
//! reads come down this same queue and re-asking one is free. Which is which is
//! [`crate::ui::Posted`]'s field, said by the control that composed it.
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
//! than with the aim — which is now true of the routed path as well
//! ([`routed`], bl-c70d): the aim is not read on this pass at all.

use std::path::Path;

use crate::channel::Reach;
use crate::state::{Link, Said};
use crate::ui::Posted;

/// Send everything the frame composed since the last pass.
///
/// **A fanned gesture is stamped by its leg** and a routed one by
/// [`crate::seat::route`], which is the one place the channel is chosen; the
/// aim is read for neither. An aim is where a gesture was *composed*, and an
/// operator may compose one aimed at a wall on one channel while a control
/// fires at a row on another — so a receipt stamped with the aim is filed under
/// a channel the gesture never went down, and a gesture composed with no aim
/// earned a stamp naming nothing at all (bl-c70d).
pub fn tick(link: &Link, root: &Path) {
    let standing = link.standing();
    for posted in link.compose() {
        if crate::envelope::workspace(&posted.envelope).is_none() {
            for held in &standing.channels {
                if let Err(reach) = super::down(link, root, held, &posted.envelope) {
                    link.heard(held, said(&posted, reach));
                }
            }
            continue;
        }
        routed(link, root, &posted);
    }
}

/// One gesture down the channel its workspace names, and the receipt filed
/// against **that** channel.
///
/// [`crate::seat::Routed`] answers the seat-side name whether or not anything
/// opened, so the failure arm files under the channel the gesture would have
/// crossed on rather than under a second guess about it.
fn routed(link: &Link, root: &Path, posted: &Posted) {
    let chosen = crate::seat::route(root, &posted.envelope);
    let asked = chosen
        .sent
        .map_err(Reach::Unsent)
        .and_then(|(open, carried)| open.ask(&carried));
    match asked {
        // A routed reply is stamped with the op it answers (bl-b180): a
        // refusal wears no `kind`, and the start held across two acts has to
        // know whether the sentence is its own. Every routed gesture is
        // stamped rather than only an act's, because the stamp is consulted
        // for exactly two ops and a second arm here for the reads nobody
        // routes today would be a branch with no beat. The fanned path stays
        // bare: it carries the window's own reads, which nothing is held
        // against.
        Ok(stream) => {
            let op = crate::envelope::op(&posted.envelope);
            for frame in stream {
                link.heard(
                    &chosen.down,
                    Said::Receipt {
                        op: op.clone(),
                        frame,
                    },
                );
            }
        }
        Err(reach) => link.heard(&chosen.down, said(posted, reach)),
    }
}

/// **Which sentence a failed leg earns**, which is the whole of what this ball
/// added to the send path.
///
/// A read that could not be answered is the channel's own relationship and is
/// asked again on the next beat regardless. An act is an exchange with no
/// second chance, so it is reported as one and carries its `op`.
fn said(posted: &Posted, reach: Reach) -> Said {
    if !posted.act {
        return Said::Unreachable(reach.said());
    }
    Said::Acted {
        op: crate::envelope::op(&posted.envelope),
        reach,
    }
}

#[cfg(test)]
mod tests;
