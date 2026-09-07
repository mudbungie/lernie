//! **The visual language's tokens**, and the one home for every colour, size
//! and gap the window spends (bl-73d2; `docs/STYLE.md` is the prose).
//!
//! The language's home is the phone seat (yog-android `docs/STYLE.md`) and
//! this file carries its bytes verbatim plus the desktop's own deltas, each
//! named where it is declared. Three rules govern it and `docs/STYLE.md`
//! states them at length: **colour means state** (six states, six accents, no
//! seventh — operator ruling 2026-09-06), **no outlines** (a block is raised by
//! a tint, never boxed), and **one module** (no pane spells a colour;
//! `rules/no-literal-colour.yml` refuses one).
//!
//! **A token this build does not know keeps its word and paints plain.** That
//! is the reply vocabulary's rung 3 carried through to the glass: an
//! unrecognised state is unstyled, never restyled as a state it is not,
//! because a colour is a claim and the wrong claim is worse than none.

use egui::Color32;

use crate::reply::convs::{AgentState, Tone};

/// The anatomy as paint: the row, the connector, the ruled block, the
/// section and the field.
pub mod paint;
/// The adapter into egui: the tokens installed as a `Style` once per frame.
mod visuals;

pub use visuals::{install, visuals};

/// **The ground ladder** — each rung brighter than the one before, asserted.
/// The glass; every pane's body; a row and a control at rest.
pub const GROUND: Color32 = Color32::from_rgb(10, 8, 15);
/// Elevation one: a field, a popup's body, a row under the pointer.
pub const SURFACE: Color32 = Color32::from_rgb(22, 20, 30);
/// Elevation two: a control while pressed or open, a selected row's tint.
pub const RAISED: Color32 = Color32::from_rgb(36, 33, 48);
/// The most a boundary may be: one point between rows or panes.
pub const HAIRLINE: Color32 = Color32::from_rgb(48, 44, 62);

/// **The ink scale.** Primary text.
pub const INK: Color32 = Color32::from_rgb(232, 230, 238);
/// Secondary: a stamp, a count, a hint, a resting row.
pub const INK_WEAK: Color32 = Color32::from_rgb(152, 148, 168);
/// A placeholder, a disabled control's words.
pub const INK_FAINT: Color32 = Color32::from_rgb(98, 94, 114);

const ATTENTION: Color32 = Color32::from_rgb(110, 222, 148);
const WORKING: Color32 = Color32::from_rgb(96, 168, 255);
const INFERENCE: Color32 = Color32::from_rgb(196, 140, 255);
const ANNOTATION: Color32 = Color32::from_rgb(255, 170, 92);
const ERROR: Color32 = Color32::from_rgb(255, 108, 132);

/// **The brand is not a seventh colour**: it is the working blue, worn where
/// the operator's own act is on the glass — the caret's field, the send, the
/// operator's message rule, the aimed and selected row.
pub const BRAND: Color32 = WORKING;

/// **The six states, and the whole of what a hue may mean.** A token names
/// the state, never the colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Asking for you: waiting on the operator, a parked call, a queue entry.
    Attention,
    /// Doing work: a tool call running, a foot executing, a step in flight.
    Working,
    /// The model is generating: the streaming tail, a turn in inference.
    Inference,
    /// High salience, neither working nor failed: a note, an interrupt, a
    /// reply in doubt, a warning a pane must say.
    Annotation,
    /// Done, archived, nothing happening — the opposite of attention.
    Rest,
    /// Failed and it will not mend itself: a refusal, a stopped driver, a
    /// wire that will not dial.
    Error,
}

/// The six, in the order the ruling names them.
pub const STATES: [State; 6] = [
    State::Attention,
    State::Working,
    State::Inference,
    State::Annotation,
    State::Rest,
    State::Error,
];

/// **The accent a state wears.** Spent as ink, as a three-point rule beside a
/// block, or as a low-alpha tint — never as a full fill behind text.
pub fn accent(state: State) -> Color32 {
    match state {
        State::Attention => ATTENTION,
        State::Working => WORKING,
        State::Inference => INFERENCE,
        State::Annotation => ANNOTATION,
        State::Rest => INK_WEAK,
        State::Error => ERROR,
    }
}

/// **The same accent at tint strength** — the alpha a block or a selected row
/// is raised by, low enough that ink over it keeps its contrast.
pub fn tint(state: State) -> Color32 {
    accent(state).gamma_multiply(TINT_ALPHA)
}

/// How much of an accent a tint is: a wash, not a fill.
const TINT_ALPHA: f32 = 0.16;

/// **The colour a refusal or an unreadable answer is said in** — the
/// annotation accent under the name every pane already spends.
pub const NOTICE: Color32 = ANNOTATION;

/// **The two inks a QR symbol is drawn in**, and they are the only pair here
/// that is not a matter of language. A symbol is defined dark-on-light and a
/// camera is what reads it, so these are black on white whatever the window's
/// visuals are — a dark-themed pane drawing its symbol in theme colours draws
/// one a phone will not lock onto. `PAPER` and `INK` because what they mean is
/// *the ground* and *the mark*, which is what a decoder is looking for.
pub const QR_PAPER: Color32 = Color32::WHITE;
pub const QR_INK: Color32 = Color32::BLACK;

/// **A conversation's state, read onto the six.** A held driver is the model
/// at work; streaming is a tool in flight (REMOTE §11's own division, and the
/// phone's `theme::tone` reads the row tone the same way); stopped will not
/// mend itself; settled is at rest. An unknown word is unstyled.
pub fn state_of(state: &AgentState) -> Option<State> {
    match state {
        AgentState::Live => Some(State::Inference),
        AgentState::InFlight => Some(State::Working),
        AgentState::Stopped => Some(State::Error),
        AgentState::Quiescent => Some(State::Rest),
        AgentState::Unknown(_) => None,
    }
}

/// The ink a conversation's badge is painted in.
pub fn state_ink(state: &AgentState) -> Color32 {
    state_of(state).map_or(INK, accent)
}

/// **The wire's row tone, read onto the six** (REMOTE §11): `good` is done
/// and so at rest, `bad` is an error, `live` is the streaming tail and so
/// inference, `in-flight` is a tool running and so working; `plain` and `weak`
/// are the ink scale, not a state.
pub fn tone_ink(tone: &Tone) -> Color32 {
    match tone {
        Tone::Live => accent(State::Inference),
        Tone::InFlight => accent(State::Working),
        Tone::Good => accent(State::Rest),
        Tone::Bad => accent(State::Error),
        Tone::Weak => INK_WEAK,
        Tone::Plain | Tone::Unknown(_) => INK,
    }
}

/// **Who is speaking, told by weight and never by hue.** Four speakers would
/// otherwise be four colours, and colour means state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Speaker {
    /// The operator's own words — the brand, because it is their act.
    Operator,
    /// The model's answer, in full ink.
    Model,
    /// A peer agent or a tool, in weak ink.
    Peer,
    /// A sender that has ended, in faint ink.
    Ended,
}

/// The ink a speaker's rule and header are painted in.
pub fn speaker(who: Speaker) -> Color32 {
    match who {
        Speaker::Operator => BRAND,
        Speaker::Model => INK,
        Speaker::Peer => INK_WEAK,
        Speaker::Ended => INK_FAINT,
    }
}

/// **Spacing, in points** — every gap on the glass is one of these.
/// **Desktop delta:** one step under the phone's smallest, for the gap between
/// lines that are read as one paragraph and for a control's padding down —
/// the phone's `XS` there put every covering pane past the narrowest shape
/// the layout promises (§4.32's density constraint), and a desktop reads
/// a stack of short lines as prose, not as a list of targets.
pub mod space {
    pub const XXS: f32 = 2.0;
    pub const XS: f32 = 4.0;
    pub const S: f32 = 8.0;
    pub const M: f32 = 12.0;
    pub const L: f32 = 16.0;
    pub const XL: f32 = 24.0;
}

/// **Type, in points** — four sizes and no fifth. **Desktop delta:** every
/// size sits two points under the phone's (15 → 13, 20 → 18), because a
/// desktop is read at arm's length and holds three columns of prose where the
/// phone holds one — and the covering panes must fit the narrowest shape the
/// layout promises (§4.32), which the phone's body overruns.
pub mod type_scale {
    pub const SMALL: f32 = 11.0;
    pub const MONO: f32 = 12.0;
    pub const BODY: f32 = 13.0;
    pub const HEADING: f32 = 18.0;
}

/// **A list row's height**, in points. **Desktop delta:** the phone's
/// 48-point touch target is a thumb's; a pointer's is 24, and a list of
/// conversations at 48 a row is a list that scrolls twice as much. A control
/// is its word plus the padding and stands no taller than it needs.
pub const ROW: f32 = 24.0;
/// A block's corner. **Desktop delta:** 6 where the phone has 10, in
/// proportion to [`ROW`].
pub const RADIUS: f32 = 6.0;
/// A state's rule beside a block or a row, in points wide.
pub const RULE: f32 = 3.0;

#[cfg(test)]
mod tests;
