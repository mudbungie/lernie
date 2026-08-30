//! **The window** — the seat's whole reason to exist (yog's `docs/REMOTE.md`
//! §7, §8.2, §9.7; DESIGN §4.11).
//!
//! # The frame renders a snapshot and does nothing else
//!
//! Every function below takes what it is given and paints it. Nothing here
//! dials, reads a file, blocks or waits: a frame that could do any of those is
//! a frame that can freeze, and a frozen window is the one failure a seat has
//! no excuse for. What fills the [`Model`] is [`crate::offframe`], and the
//! frame's whole side of it is one `settle` at the top of an update.
//!
//! The one thing a frame *produces* is [`Model::outbox`]: the gestures a click
//! or a keystroke composed, drained by whoever can send them. A frame that
//! posted its own gesture would be a frame that waits.
//!
//! # It fires through the verb table, not beside it
//!
//! A composed gesture is built by [`crate::verbs`] — the same rows
//! `lernie message` spends. So the window and the command line are two
//! serializations of one surface rather than two spellings of a gesture, which
//! is REMOTE §3's rule read one more time.
//!
//! # Every assertion about this module goes through the paint probe
//!
//! `crate::paint_probe` is the one walk and
//! `rules/no-hand-rolled-paint-walk.yml` is why. A galley reports the string
//! that went IN, so a label the toolkit elided to `…` reads back whole and
//! every assertion against it is blind to truncation. That rule arrived here
//! with this window, as a rule and not as a memory.

/// The chat pane: one conversation, entry by entry.
pub mod chat;
/// The composer: what an operator types, and the gesture it becomes.
pub mod composer;
/// The conversation list.
pub mod convs;
/// What the window holds between frames, and how a reply becomes part of it.
pub mod model;
/// The roster: every workspace this seat can reach, grouped by channel.
pub mod roster;
/// The layout, and the notice that stands where content would have been.
pub mod shell;
/// The ink a row is painted in.
pub mod theme;

pub use model::{Aim, Channel, Chunk, Model, Notice};
pub use shell::render;
