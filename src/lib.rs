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
//! and the [`reply`] vocabulary — the typed half a window paints from, read
//! back out of the answers those channels carry.
//!
//! **What is not built, and it is the larger half**: the window. The seat's
//! whole reason to exist is a graphical face, and this crate is the transport
//! that face will stand on. DESIGN §6 is the ledger of what that costs — the
//! off-frame threads, the paint layer — and every row of it is a filed ball,
//! not a hand-wave.

pub mod channel;
pub mod cli;
pub mod envelope;
pub mod paths;
pub mod reply;
pub mod seat;

/// Scaffolding the suite shares. Never compiled into a released binary.
#[cfg(test)]
pub(crate) mod test_support;
