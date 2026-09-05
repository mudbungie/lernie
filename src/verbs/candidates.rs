//! **The candidate family** (yog's `docs/REMOTE.md` §9.7; VISION §4.10) — the
//! three ops of the n-attempt path: spread one prepared start over n isolated
//! attempts, accept one of what comes back, release the rest.
//!
//! # Three doors and no rows, and the reason is the fan bl-4855 named
//!
//! `deliver` and `retire` carry nothing but named strings, so by [`super`]'s
//! rule they could be rows — and they must not be. A row is a word an operator
//! TYPES, and neither of these names a workspace anywhere in its envelope
//! (`crate::envelope`), so `lernie deliver …` would be **fanned** down every
//! channel this box holds (`crate::seat::fan`): one accepted candidate on
//! every engine the operator is a client of. That is the hazard DESIGN §4.30
//! settled for the config write, and the settlement there was that a gesture
//! about ONE engine says which — a thing a control can do, because it fired
//! from a pane open on an aim, and argv cannot, because it has no row for it.
//! So the window composes these and `lernie ask` stays the door argv has.
//!
//! `fan` could never have been a row anyway: it carries a **prepared body** and
//! a count, and a nested object is not a word an operator types
//! ([`super::start`]'s rule, fourth application).
//!
//! # The ball is always named, because this seat has no focus at the far end
//!
//! Every one of the three takes an optional `ball` upstream, and omitting it
//! means *the engine's own focused ball*. A seat has no such focus — its
//! gestures are composed off a row that already names one — so all three
//! spell it. The frames that omit it are recorded as unemitted in
//! `src/verbs/tests/corpus/emits.rs`, by count and reason.

use serde_json::{Value, json};

use crate::envelope;
use crate::reply::start::Prepared;

/// Spread a prepared start over n isolated attempts.
pub const FAN: &str = "fan";
/// Accept one candidate onto the obligation's own target.
pub const DELIVER: &str = "deliver";
/// Release a candidate's worktree.
pub const RETIRE: &str = "retire";

/// The obligation's two halves — a target is both or neither, upstream's own
/// shape.
const BALL: &str = "ball";
const PROJECT: &str = "project";
/// balls' opaque attempt handle, as the work diff read it back.
const HANDLE: &str = "handle";
/// The delivery subject, verbatim.
const SUMMARY: &str = "summary";
/// How many attempts.
const N: &str = "n";

/// **Spread a prepared start over `n` candidates** off one pinned tip of the
/// ball's delivery target.
///
/// The prepared body is handed back verbatim with one field rewritten — the
/// workspace, into this box's spelling — for [`super::start::prompt`]'s reason
/// exactly: it came back in the host's, and §8.2's mapping is spent at
/// `crate::seat::route` and nowhere else. `fan` is one of the two envelopes
/// `crate::envelope` reads a workspace out of a nested body for, so it routes
/// like any addressed gesture.
pub fn fan(prepared: &Prepared, address: String, ball: String, project: String, n: u64) -> Value {
    json!({
        envelope::OP: FAN,
        envelope::PREPARED: envelope::with_workspace(&prepared.body, &address),
        BALL: ball,
        PROJECT: project,
        N: n,
    })
}

/// **Accept one candidate**: the ordinary recursive delivery of its attempt
/// onto the ball's own `work/<id>` ref. `summary` becomes the delivery
/// subject, which balls tags with the handle — the only acceptance mark there
/// is, derived from the target's history rather than stored.
pub fn deliver(ball: String, project: String, handle: String, summary: String) -> Value {
    json!({
        envelope::OP: DELIVER,
        BALL: ball,
        PROJECT: project,
        HANDLE: handle,
        SUMMARY: summary,
    })
}

/// **Release a candidate's worktree.** Its source ref stays addressable unless
/// this project's declared retention says the keep has expired; the reply says
/// which happened, and this seat predicts neither.
pub fn retire(ball: String, project: String, handle: String) -> Value {
    json!({
        envelope::OP: RETIRE,
        BALL: ball,
        PROJECT: project,
        HANDLE: handle,
    })
}

#[cfg(test)]
mod tests;
