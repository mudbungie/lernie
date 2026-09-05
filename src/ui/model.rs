//! **What the window holds between frames**, and the one door a reply comes in
//! through.
//!
//! This file is the module list and the re-export surface; the struct itself
//! is [`held`], split out at the 300-line cap on the one seam left here. The
//! two change for different reasons: a pane that learns to hold something
//! grows the struct, and a pane that learns to DO something grows the list.
//!
//! Nothing here paints and nothing here dials. It is the snapshot a frame reads
//! and the rules for changing it, so every one of them is a pure function a
//! test reads back as a value.
//!
//! **[`Model::absorb`] is the single door, and it drops nothing.** A frame that
//! arrives is either content this build paints or a [`Notice`] the shell paints
//! *where that content would have been* — which is the reply vocabulary's own
//! policy (`crate::reply`, rung 2) honoured on the screen rather than only in
//! the type: a refusal and an unreadable frame are both visible rows, and
//! neither is a silent drop.

/// The one door a reply comes in through, and the leg that brought none.
mod absorb;
/// What a control does, whichever control did it.
mod acts;
/// Which wall the window is aimed at, and the two questions asked of a name.
mod aim;
/// The ball pane between frames: the boards, the bindings, and the wall's own.
mod board;
/// What a channel is, and what a gesture aimed down one must be addressed as.
mod channel;
/// The claim a start leaves on the selection, and the row it stands in for.
mod claim;
/// The clients pane between frames: whether it is open, and what it filed.
mod clients;
/// The config pane between frames: which file it is pointed at.
mod config;
/// The deeper records between frames: the one read the pane posts.
mod deep;
/// An enrollment, between the control that opened it and the symbol it ends at.
mod enroll;
/// Which composer box a row menu's navigation asked for the cursor in.
mod fill;
/// The fleet pane between frames: three words, five acts and two reads.
mod fleet;
/// **What the window holds between frames** — the struct itself.
mod held;
/// The three panes that are pure listings, and the field that says which is up.
mod listing;
/// The login pane between frames: what it asks about, and the acts it spends.
mod login;
/// What the seat last heard that was not content, and how the shell says it.
mod notice;
/// What a frame composed, and what a lost reply would mean for it.
mod posted;
/// The decision queue between frames: what is asking, and the acts on a row.
mod queue;
/// The records pane between frames: open or not, and what its two reads filed.
mod records;
/// The spine's own state: the draft a fork is composed from.
mod spine;
/// A start, between its two acts.
mod start;
/// The trail pane between frames: open or not, and what each channel has done.
mod trail;
/// The tuning pane between frames, and the four acts its controls spend.
mod tuning;
/// An unmaking between frames: its subject, its arming, and whether it is asked.
mod unmake;
/// The window's own two panes: the engines' verb table, and what a needle found.
mod window;

pub use aim::Aim;
pub use board::block::Authoring;
pub use board::{Bindings, Columns};
pub use channel::{Channel, Chunk, Held};
pub use config::Configuring;
pub use enroll::{Enrolling, Grade, Shown};
pub use fill::Fill;
pub use fleet::{Armed, Fleet};
pub use held::Model;
pub use listing::Listing;
pub use login::Login;
pub use notice::Notice;
pub use posted::Posted;
pub use queue::Asking;
pub use records::Records;
pub use spine::Forking;
pub use start::{Phase, Start};
pub use trail::Trail;
pub use tuning::{Edit, Tuning};
pub use unmake::Unmaking;
pub use window::{Hits, Lookup, Pages};

#[cfg(test)]
mod tests;
