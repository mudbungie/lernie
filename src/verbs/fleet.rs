//! **The fleet family** — running a workspace's ready balls, watching what its
//! agents do, and reading what they changed (yog's `docs/REMOTE.md` §9.7;
//! bl-a43a).
//!
//! # Two families, two carriers, and one reply kind between them
//!
//! `fleet` and `disband` are the **fleet loop**: a workspace claims its
//! project's top ready ball and starts a drone on it, up to a cap. `arm` and
//! `disarm` are the **alignment monitor**: a cheap model reads each commit
//! against its goal and records a verdict on the trail. They are separate
//! settings on separate carriers — and they answer with the SAME `armed`
//! reply. So a seat can never tell them apart by the answer, and every reader
//! of one reads the `op` back instead (DESIGN §4.33). Nothing in this file
//! names the reply at all, which is the shape of that rule rather than a note
//! about it.
//!
//! # Five rows and one door, because `cap` is a number
//!
//! [`super`]'s table is rows of named strings and refuses to grow an arm for
//! anything else, so `fleet` — which carries a cap — is a typed door beside
//! the trail's and the tuning pair's, on exactly their terms. The other five
//! are named strings and are rows.
//!
//! # `work-diff` is a row because the seat composes only its bare form
//!
//! The op takes an optional `file` object naming a ball and a path, and that
//! form answers a patch instead of a listing. This seat composes the bare
//! listing, exactly as it composes the bare `files` read — a control that
//! guessed at a file would answer a question nobody asked — so the row's
//! parameters are the workspace and nothing else, and the two file forms are
//! recorded in `src/verbs/tests/corpus/emits.rs` by count and reason.

use serde_json::{Value, json};

use super::Verb;
use crate::envelope;

/// The word the loop's door spells, and the envelope's `op`. One fact.
pub const FLEET: &str = "fleet";
/// The field that says how many at once.
const CAP: &str = "cap";
/// The field that says whose ready balls.
const PROJECT: &str = "project";

/// **Run this workspace's ready balls, up to `cap` at once.**
pub fn fleet(workspace: String, project: String, cap: u64) -> Value {
    json!({ envelope::OP: FLEET, envelope::WORKSPACE: workspace, PROJECT: project, CAP: cap })
}

/// Stop the loop.
pub const DISBAND: Verb = Verb {
    word: "disband",
    params: &["workspace"],
    summary: "stop running a fleet in this workspace",
    detail: "Removes the workspace's fleet setting. Nothing further is \
             claimed, started or released; everything already running is \
             untouched and keeps its ball. Its own verb rather than a cap of \
             zero, which is an armed loop that spawns nothing and still reaps.",
};

/// Raise the alignment monitor.
pub const ARM: Verb = Verb {
    word: "arm",
    params: &["workspace", "model"],
    summary: "watch this workspace's agents with a cheap model",
    detail: "Arms the alignment monitor on the named workspace, pinned to the \
             model you name. From then on, whenever an agent commits, one \
             small tool-less call reads its goal and the work since the last \
             check and answers aligned, drifting or diverged with one \
             sentence — recorded on the ops trail, which is the only thing it \
             does by default. It costs money per check; `disarm` ends it. It \
             is NOT the fleet loop: that is `fleet` and `disband`, and the two \
             answer with the same receipt.",
};

/// Drop it.
pub const DISARM: Verb = Verb {
    word: "disarm",
    params: &["workspace"],
    summary: "stop watching this workspace",
    detail: "Removes the workspace's monitor setting. No further checks are \
             made and nothing further is charged; every verdict already \
             recorded stays on the trail.",
};

/// Flush the inboxes.
pub const SCAN: Verb = Verb {
    word: "scan",
    params: &["workspace"],
    summary: "flush this workspace's inboxes now",
    detail: "Delivers whatever is waiting in the workspace's conversations \
             rather than waiting for the next beat to notice it. It answers \
             with a captured run.",
};

/// The delivery attempts.
pub const SCIENCE: Verb = Verb {
    word: "science",
    params: &["workspace"],
    summary: "every delivery attempt of this workspace, with what it cost",
    detail: "One row per attempt — the ordinary claim and each fan candidate \
             alike: the goal it was fired with, the documents frozen onto its \
             dispatch commit, the config commit that governs it, both ends of \
             its project diff, its tokens and wall seconds, what it last said, \
             every message delivered into it, and how it ended. Everything is \
             derived when you ask, so the same row a minute later is a \
             statement about the world a minute later.",
};

/// What the agents changed.
pub const WORK_DIFF: Verb = Verb {
    word: "work-diff",
    params: &["workspace"],
    summary: "what this workspace's agents changed in their project",
    detail: "For every ball the workspace holds, the changes on that ball's \
             work branch that are not yet on the branch it delivers into — one \
             row per changed file, with lines added and removed. A repository \
             it cannot read, and a branch that is not there yet, are each said \
             outright rather than shown as an empty list. The form that names \
             a ball and a path and answers that one file's patch is not \
             composed here.",
};

/// Stop the loop, typed.
pub fn disband(workspace: String) -> Value {
    DISBAND.built(vec![workspace])
}

/// Raise the monitor, typed.
pub fn arm(workspace: String, model: String) -> Value {
    ARM.built(vec![workspace, model])
}

/// Drop it, typed.
pub fn disarm(workspace: String) -> Value {
    DISARM.built(vec![workspace])
}

/// Flush the inboxes, typed.
pub fn scan(workspace: String) -> Value {
    SCAN.built(vec![workspace])
}

/// The attempts, typed.
pub fn science(workspace: String) -> Value {
    SCIENCE.built(vec![workspace])
}

/// What changed, typed.
pub fn work_diff(workspace: String) -> Value {
    WORK_DIFF.built(vec![workspace])
}

#[cfg(test)]
mod tests;
