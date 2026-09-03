//! **The role-tuning family** (yog's `docs/REMOTE.md` §3; PROTOCOL 6) — one
//! read that says what a workspace's roles are set to, and the three writes
//! that set them.
//!
//! They are one module because they are one subject read from both ends: the
//! engine writes all three into the same `providers.yaml` assignment on the
//! same config lineage, and `roles` reads that assignment back. yog's own help
//! row for `roles` states the pairing — *"This is what `/model`, `/effort` and
//! `/priority` have set — read back from the same place they write it, so a
//! control can open showing what is in force instead of blank."* A seat that
//! filed the read with the other reads and the writes with the other writes
//! would have split the one fact into two places.
//!
//! # Two of them are rows and two are doors without rows
//!
//! [`super`]'s table is *a word and its parameters, all of them named
//! strings*, which is what keeps it one builder with no per-verb arm to drift.
//! [`ROLES`] and [`MODEL`] fit it exactly and are rows. The other two do not,
//! and [`super`]'s own rule says what happens then — such a gesture *"is not
//! added as a special case"*:
//!
//! - `effort` carries `level` as a string **or `null`**, and null is the whole
//!   of what `off` means. yog spells it that way on purpose: *"`off` removes
//!   the line, which is the only way to say no level: absent means none
//!   requested and the provider's own default governs, so there is no third
//!   state to write."* A row would have to send the word `"off"` across, which
//!   is a fifth level the boundary refuses by name.
//! - `priority` carries `on` as a **bool**. It is a checkbox rather than a
//!   choice of lanes, and `false` is the provider's own default lane.
//!
//! So they are typed doors beside the rows, exactly as [`super::start`]'s pair
//! are — same rule, second application.
//!
//! # What the writes answer is a captured run, and this seat already reads one
//!
//! All three writes end in one `litany config` drive and answer
//! [`crate::reply::Reply::Outcome`] — yog states it as *"one staging path, one
//! `litany config` drive, one `Reply::Outcome`, so a tuning gesture and a pick
//! fail the same way."* Nothing new is decoded for them: a refusal arrives in
//! the child's own words on the surface that already paints one.

use serde_json::{Value, json};

use super::Verb;
use crate::envelope;

/// **The read**: one row per role this workspace's config declares.
pub const ROLES: Verb = Verb {
    word: "roles",
    params: &["workspace"],
    summary: "what this workspace's roles are set to, and how each is tuned",
    detail: "One row per role the workspace's config declares: the provider \
             row and model id bound to it, the effort level it asks for, and \
             whether it asks for the priority lane. It reads back from the \
             same assignment `model`, `effort` and `priority` write, so a \
             control opens showing what is in force rather than blank. Under \
             follow-the-tip these are the settings every conversation here \
             resolves at its next step, not only the next one started. A \
             workspace whose config declares no role answers an empty list \
             rather than refusing — nothing set is a state a fresh workspace \
             is really in.",
};

/// **The assignment**: which provider row and model id a role runs on.
pub const MODEL: Verb = Verb {
    word: "model",
    params: &["workspace", "role", "provider", "model"],
    summary: "give a role this model, on the workspace's config lineage",
    detail: "One write into `providers.yaml` on the workspace's default \
             config lineage, through `litany config`. It reaches the \
             conversations already running: each follows its lineage's head at \
             every step boundary, so this governs the next step of every \
             conversation on the wall and not only the next one started. A \
             provider row the engine does not have, and a row whose dialect \
             cannot carry a turn, are each refused before anything is written.",
};

/// The tuning writes that are doors rather than rows — their `op`, spelled
/// once, because the tag on a control and the envelope it composes must be one
/// string (`crate::ui::act`).
pub const EFFORT: &str = "effort";
pub const PRIORITY: &str = "priority";

/// The field each door carries beyond the role and the wall.
const LEVEL: &str = "level";
const ON: &str = "on";
/// The role the tuning is about — a field name both doors share with
/// [`MODEL`]'s row, so it is spelled here and read from the row's `params`
/// there.
const ROLE: &str = "role";

/// **What a workspace's roles are**, asked of the wall `address` names.
pub fn roles(workspace: String) -> Value {
    ROLES.built(vec![workspace])
}

/// **Give `role` a model**, on that workspace's config lineage.
pub fn model(workspace: String, role: String, provider: String, model: String) -> Value {
    MODEL.built(vec![workspace, role, provider, model])
}

/// **Ask `role`'s model calls for this much reasoning**, or for none.
///
/// `None` is the wire's `null`, and it is the only way to say *no level*: it
/// removes the line, and absence means the provider's own default governs.
/// There is no word for it on the wire, so there is no word for it here.
pub fn effort(workspace: String, role: String, level: Option<String>) -> Value {
    json!({
        envelope::OP: EFFORT,
        envelope::WORKSPACE: workspace,
        ROLE: role,
        LEVEL: level,
    })
}

/// **Turn `role`'s priority lane on or off.**
pub fn priority(workspace: String, role: String, on: bool) -> Value {
    json!({
        envelope::OP: PRIORITY,
        envelope::WORKSPACE: workspace,
        ROLE: role,
        ON: on,
    })
}

/// **The four levels a control offers**, in the order it offers them — the
/// wire's three words and the absence, which is the fourth thing an operator
/// can choose and is not a word at all.
///
/// A `None` in the list rather than a fourth string, because the envelope's
/// `level` is `Option<String>` and a list of words would need a translation
/// table to get back to it — which is the one thing `crate::ui::act` and this
/// module both exist not to have.
pub fn levels() -> Vec<Option<String>> {
    vec![
        Some("low".to_owned()),
        Some("medium".to_owned()),
        Some("high".to_owned()),
        None,
    ]
}

/// **What a level reads as on a control.** The absence is the one that needs a
/// word, and `off` is yog's own spelling of it in the `effort` usage line.
pub const OFF: &str = "off";

/// One level's word.
pub fn word(level: Option<&String>) -> String {
    level.map_or_else(|| OFF.to_owned(), Clone::clone)
}

#[cfg(test)]
mod tests;
