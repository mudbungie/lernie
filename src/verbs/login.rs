//! **The sign-in family** (yog's `docs/REMOTE.md` §8.3; PROTOCOL 13) — the
//! table a wall can sign in to, what one row offers, the act that starts a
//! sign-in, and the lane that streams it.
//!
//! They are one module for [`super::tuning`]'s reason: one subject, read and
//! written from both ends. `providers` is the table, `login` starts a run
//! **inside that workspace's wall on the engine**, and `login-tail` is that
//! same run held open. A seat that filed the read with the other reads and the
//! act with the other acts would have split the one subject in four.
//!
//! # All four are rows, and the act is a row like any other
//!
//! Every parameter is a named string, which is [`super`]'s whole test for a
//! row — so nothing here needs a typed door beside the table the way the
//! tuning pair do. The act is not special-cased either: `login` crosses the
//! §8.5 boundary as a gesture, and what makes it an ACT rather than a read is
//! recorded at the control that composes it (`crate::ui::model::posted`), not
//! in this table.
//!
//! # The act never waits, and neither does this end
//!
//! Upstream answers `login` at once with the run's standing rather than with
//! the flow's outcome — *"an act that waited out a browser-minutes flow would
//! stop every deposit converging"* — so its receipt and the lane's first frame
//! are the same value, decoded by the same reader (`crate::reply::login`).
//!
//! # `login-tail` is follow-class, which is why it has its own thread
//!
//! It is answered at the provider's pace rather than the asker's, so putting
//! it in the serial pass would stall every other read behind a question that is
//! supposed to stay open. `crate::offframe::signin` is that thread, and it is
//! `crate::offframe::follow`'s shape one noun over.

use serde_json::Value;

use super::Verb;

/// **The table**: every provider row this workspace's wall routes.
pub const PROVIDERS: Verb = Verb {
    word: "providers",
    params: &["workspace"],
    summary: "what this workspace's wall can sign in to",
    detail: "One row per provider the wall routes, in the engine's own \
             listing order — which is brazen's routing order, so the first \
             row that can answer is the one a turn goes down. Each says what \
             the engine knows about its credential, why a sign-in cannot be \
             started on it where one cannot, and whether it takes an effort \
             level and the priority lane at all. It is read on the wall the \
             workspace names, so a wall held elsewhere answers about the \
             credentials its own agents read.",
};

/// **The offering**: what one row will answer to.
pub const MODELS: Verb = Verb {
    word: "models",
    params: &["workspace", "provider"],
    summary: "the model ids one provider row offers",
    detail: "The ids that row answers to, as the engine has them — the \
             values `model` takes as its last argument. It is asked of one \
             row rather than of the wall, because two rows offering the same \
             id are still two different routes to it.",
};

/// **The act**: start a sign-in on one row, inside that workspace's wall.
pub const LOGIN: Verb = Verb {
    word: "login",
    params: &["workspace", "provider"],
    summary: "start a sign-in on one provider row, in that workspace's wall",
    detail: "The run happens on the ENGINE, inside the named workspace's \
             wall, so the credential lands where the agents that need it run \
             — nothing credential-shaped ever crosses this wire. It answers \
             at once with the run's standing rather than waiting the flow \
             out, and the lines are read with `login-tail`. A second sign-in \
             on a live pair replaces the first, which is the operator's own \
             restart and the reason there is no cancel. A row whose flow \
             needs a browser at the engine's own loopback can only be \
             completed from a browser on that box, or through a port-forward \
             the operator sets up.",
};

/// **The lane**: everything one sign-in has said, held open.
pub const LOGIN_TAIL: Verb = Verb {
    word: "login-tail",
    params: &["workspace", "provider"],
    summary: "hold the line on one sign-in's output",
    detail: "Buffered from the start, then live to the outcome, with the \
             settled exit as the last frame. Each frame carries what the run \
             said SINCE the last one, so a read starts from empty and asking \
             twice replays the whole of it. A pair nobody has signed in to \
             answers one empty frame — nobody has signed in here is a \
             reading, not a silence.",
};

/// **The provider table**, asked of the wall `address` names.
pub fn providers(workspace: String) -> Value {
    PROVIDERS.built(vec![workspace])
}

/// **What one row offers.**
pub fn models(workspace: String, provider: String) -> Value {
    MODELS.built(vec![workspace, provider])
}

/// **Start a sign-in** on one row of that wall.
pub fn login(workspace: String, provider: String) -> Value {
    LOGIN.built(vec![workspace, provider])
}

/// **Hold the line on one sign-in's output.**
pub fn login_tail(workspace: String, provider: String) -> Value {
    LOGIN_TAIL.built(vec![workspace, provider])
}
