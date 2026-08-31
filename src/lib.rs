//! lernie — the **seat**: the operator's face on a yog server.
//!
//! A seat holds an operator-issued certificate for this box and dials in to an
//! engine over mTLS. It asks the boundary's queries, dispatches its actions,
//! and paints what comes back. It holds no world, runs no agent, and executes
//! nothing — everything durable is the server's (yog's `docs/REMOTE.md` §6),
//! and every execution is a foot's (§5.4).
//!
//! **THE NAME HAS TWO ERAS AND THE VERSION IS THE FENCE.** `lernie` through
//! **0.0.x** was the agent-loop engine; that program continues under the name
//! **`litany`**. `lernie` at **0.1.0 and above** is this crate. REMOTE §12
//! adopted the split and states the rule: read every `lernie` against it — a
//! bare one names the seat, and one bound to a `0.0.x` version names the
//! engine at that release.
//!
//! `docs/DESIGN.md` states the role, the module map and what is deferred.
//! yog's `docs/REMOTE.md` is the **protocol authority** this crate implements
//! against; where this crate and that document disagree, one of them is a bug,
//! and there is never a third answer.
//!
//! **What is built** (DESIGN §5): the command line ([`cli`]); the [`channel`] —
//! the mTLS dial, the version preface, the framing, and the entries an operator
//! filed; the [`envelope`] a gesture crosses in, read only as far as routing it
//! requires; the [`seat`] that resolves which engine a gesture reaches and
//! spends the one name mapping on the way, over a data root [`paths`] names;
//! the [`reply`] vocabulary — the typed half a window paints from, read
//! back out of the answers those channels carry; and the [`verbs`] an operator
//! types instead of writing one out by hand.
//!
//! And the [`ui`]: the window itself — the roster, the conversation list, the
//! chat pane and the composer, painted from a snapshot and firing gestures
//! through the same verb table the command line spends. **The frame never
//! dials.** What fills it is [`offframe`], three threads that meet the frame at
//! the one lock [`state`] holds: the asker over the standing question set, the
//! poster draining what a click composed, and the follow lane holding one
//! connection open on the focused conversation.

pub mod channel;
pub mod cli;
pub mod envelope;
/// The seat's own mark, and where a desktop actually looks for one.
pub mod mark;
/// The off-frame threads: the asker, the poster and the follow lane.
pub mod offframe;
pub mod paths;
/// Where the seat was pointed, remembered between runs.
pub mod place;
pub mod reply;
pub mod seat;
/// The link the frame and the threads share, and the crate's one lock.
pub mod state;
/// The window: the seat's face, and the paint layer that can testify about it.
pub mod ui;
pub mod verbs;

/// The one paint walk, and the harness around it. Never in a released binary.
#[cfg(test)]
pub(crate) mod paint_probe;

/// Scaffolding the suite shares. Never compiled into a released binary.
#[cfg(test)]
pub(crate) mod test_support;
