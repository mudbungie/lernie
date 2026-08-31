//! **The ink a row is painted in**, and nothing else.
//!
//! One home for every colour the window spends, so a tone means one thing on
//! every pane. The palette is small on purpose: the seat's job is to make a
//! roster of conversations legible at a glance, and a legend the operator has
//! to learn is a worse answer than four words they can read.
//!
//! **A token this build does not know keeps its word and paints plain.** That
//! is the reply vocabulary's rung 3 carried through to the glass: an
//! unrecognised state is unstyled, never restyled as a state it is not, because
//! a colour is a claim and the wrong claim is worse than none.

use egui::Color32;

use crate::reply::convs::{AgentState, Tone};

/// A conversation that is doing something right now.
const LIVE: Color32 = Color32::from_rgb(0x7f, 0xd1, 0x8f);
/// A conversation that is streaming.
const IN_FLIGHT: Color32 = Color32::from_rgb(0x7f, 0xb6, 0xd1);
/// A conversation that has stopped.
const STOPPED: Color32 = Color32::from_rgb(0xd1, 0x8f, 0x7f);
/// Something that went well.
const GOOD: Color32 = Color32::from_rgb(0x8f, 0xd1, 0x7f);
/// Something that did not.
const BAD: Color32 = Color32::from_rgb(0xd1, 0x7f, 0x7f);
/// Settled, provisional, or a word this build has never seen.
const PLAIN: Color32 = Color32::from_rgb(0xc8, 0xc8, 0xc8);
/// Weaker than plain, for a row that has said nothing yet.
const WEAK: Color32 = Color32::from_rgb(0x88, 0x88, 0x88);
/// The colour a refusal or an unreadable answer is said in.
pub const NOTICE: Color32 = Color32::from_rgb(0xe0, 0xa0, 0x60);

/// **The two inks a QR symbol is drawn in**, and they are the only pair here
/// that is not a matter of taste. A symbol is defined dark-on-light and a
/// camera is what reads it, so these are black on white whatever the window's
/// visuals are — a dark-themed pane drawing its symbol in theme colours draws
/// one a phone will not lock onto. They are named `PAPER` and `INK` rather than
/// `WHITE` and `BLACK` because what they mean is *the ground* and *the mark*,
/// which is what a decoder is looking for.
pub const PAPER: Color32 = Color32::WHITE;
pub const INK: Color32 = Color32::BLACK;

/// The ink a conversation's badge is painted in.
pub fn state_ink(state: &AgentState) -> Color32 {
    match state {
        AgentState::Live => LIVE,
        AgentState::InFlight => IN_FLIGHT,
        AgentState::Stopped => STOPPED,
        AgentState::Quiescent | AgentState::Unknown(_) => PLAIN,
    }
}

/// The ink a row's own tone is painted in.
pub fn tone_ink(tone: &Tone) -> Color32 {
    match tone {
        Tone::Live => LIVE,
        Tone::InFlight => IN_FLIGHT,
        Tone::Good => GOOD,
        Tone::Bad => BAD,
        Tone::Weak => WEAK,
        Tone::Plain | Tone::Unknown(_) => PLAIN,
    }
}

/// The window's visuals: dark, because a seat is looked at for hours beside a
/// terminal that already is.
pub fn visuals() -> egui::Visuals {
    egui::Visuals::dark()
}

#[cfg(test)]
mod tests;
