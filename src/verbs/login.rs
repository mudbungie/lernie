//! **The sign-in family** (yog's `docs/REMOTE.md` §8.3, §9.27; PROTOCOL 13) —
//! the table a workspace routes, the catalog it can sign in to, what one row
//! offers, the act that starts a sign-in, and the lane that streams it.
//!
//! They are one module for [`super::tuning`]'s reason: one subject, read and
//! written from both ends. `providers` is the routing table, `catalog` is the
//! sign-in catalog over the same row shape (bl-8c03), `login` starts a run
//! **inside that workspace's directory on the server**, and `login-tail` is that
//! same run held open. A seat that filed the read with the other reads and the
//! act with the other acts would have split the one subject in five.
//!
//! # All five are rows, and the act is a row like any other
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

/// **The table**: every provider row this workspace routes.
pub const PROVIDERS: Verb = Verb {
    word: "providers",
    params: &["workspace"],
    flags: &[],
    summary: "the provider rows this workspace routes to",
    detail: "One row per provider the workspace routes — the rows named in \
             the brazen config plus the built-in presets holding a credential \
             — in the server's own listing order, which is brazen's routing \
             order, so the first row that can answer is the one a turn goes \
             down. Each says what the server knows about its credential, why \
             a sign-in cannot be started on it where one cannot, and whether \
             it takes an effort level and the priority lane at all. A preset \
             nobody named and nobody signed into is not here; `catalog` lists \
             what can be signed into. It is read on the workspace the \
             workspace names, so a workspace held elsewhere answers about the \
             credentials its own agents read.",
};

/// **The catalog**: every provider row this workspace can sign in to.
pub const CATALOG: Verb = Verb {
    word: "catalog",
    params: &["workspace"],
    flags: &[],
    summary: "the providers this workspace can sign in to",
    detail: "One row per provider `login` can serve on this workspace, signed \
             in or not — every row carrying a sign-in flow, built in or \
             declared — in `providers`' own row shape, so a row that is \
             not signed in here is the row that appears there once it is. \
             It is the login pane's listing; a fresh workspace routes nothing \
             and can still sign in to what is here.",
};

/// **The offering**: what one row will answer to.
pub const MODELS: Verb = Verb {
    word: "models",
    params: &["workspace", "provider"],
    flags: &[],
    summary: "the models one provider offers",
    detail: "The ids that row answers to, as the server has them — the \
             values `model` takes as its last argument. It is asked of one \
             row rather than of the workspace, because two rows offering the same \
             id are still two different routes to it.",
};

/// **The act**: start a sign-in on one row, inside that workspace's directory.
pub const LOGIN: Verb = Verb {
    word: "login",
    params: &["workspace", "provider"],
    flags: &[],
    summary: "sign this workspace in to one provider",
    detail: "The run happens on the SERVER, inside the named workspace's \
             workspace, so the credential lands where the agents that need it run \
             — nothing credential-shaped ever crosses this wire. It answers \
             at once with the run's standing rather than waiting the flow \
             out, and the lines are read with `login-tail`. A second sign-in \
             on a live pair replaces the first, which is the operator's own \
             restart and the reason there is no cancel. A row whose flow \
             needs a browser at the server's own loopback can only be \
             completed from a browser on that box, or through a port-forward \
             the operator sets up.",
};

/// **The lane**: everything one sign-in has said, held open.
pub const LOGIN_TAIL: Verb = Verb {
    word: "login-tail",
    params: &["workspace", "provider"],
    flags: &[],
    summary: "hold the line on one sign-in's output",
    detail: "Buffered from the start, then live to the outcome, with the \
             settled exit as the last frame. Each frame carries what the run \
             said SINCE the last one, so a read starts from empty and asking \
             twice replays the whole of it. A pair nobody has signed in to \
             answers one empty frame — nobody has signed in here is a \
             reading, not a silence.",
};

/// **The provider table**, asked of the workspace `address` names.
pub fn providers(workspace: String) -> Value {
    PROVIDERS.built(vec![workspace], &[])
}

/// **The sign-in catalog**, asked of the same workspace.
pub fn catalog(workspace: String) -> Value {
    CATALOG.built(vec![workspace], &[])
}

/// **What one row offers.**
pub fn models(workspace: String, provider: String) -> Value {
    MODELS.built(vec![workspace, provider], &[])
}

/// **Start a sign-in** on one row of that workspace.
pub fn login(workspace: String, provider: String) -> Value {
    LOGIN.built(vec![workspace, provider], &[])
}

/// **Hold the line on one sign-in's output.**
pub fn login_tail(workspace: String, provider: String) -> Value {
    LOGIN_TAIL.built(vec![workspace, provider], &[])
}
