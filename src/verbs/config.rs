//! **The config family** (yog's `docs/REMOTE.md` §9, §9.18; PROTOCOL 13) — the
//! lineages a workspace holds, and one config file's bytes.
//!
//! They are one module for [`super::tuning`]'s reason: one subject asked at two
//! depths. `lineages` is the listing of policy branches and the paths each tip
//! holds, and `config` reads one of those paths — or one of the engine's own
//! three global files — as the typed thing it is.
//!
//! # `lineages` is a row and `config` cannot be one
//!
//! [`super`]'s table is *a word and its parameters, all of them named
//! strings*. `lineages` fits and is a row. `config`'s destination is a **nested
//! object** — `target`, carrying between one and five fields across five
//! destination shapes — so it is a typed door beside the table, exactly as
//! [`super::start`]'s pair and [`super::tuning`]'s two are. Same rule, third
//! application.
//!
//! # The read and the write are ONE op, and the discriminator is upstream's
//!
//! yog reads a `config` gesture as a **write** when it carries `text` and as a
//! **read** when it does not (REMOTE §8.5's query/action split, bl-0164). So
//! there is no `read-config` word to spell, and [`config`] and [`write`] are
//! one envelope and one field apart — which is why they are one function's
//! output with an `Option` rather than two builders that could disagree about
//! a destination.
//!
//! # Two of the five destinations name a workspace and three name an ENGINE
//!
//! `brazen` is one workspace's own `config.toml` (§16.2's per-workspace wall)
//! and `branch` is one file on one of that workspace's config lineages; both
//! carry `workspace` inside `target`, which `crate::envelope` already reads as
//! the gesture's address — so §8.2's routing and its rename apply to them
//! untouched. The other three — litany's global models file, its workflows and
//! yog's own `cadence.yaml` — belong to the ENGINE and name no workspace at
//! all, so what they address is a **channel**, and the pane asks them down the
//! channel the window is aimed at (DESIGN §4.30) rather than letting a
//! workspace-less gesture fall through to this box's own engine.

use serde_json::{Map, Value};

use super::Verb;
use crate::envelope;

/// **The listing**: every config lineage one workspace holds.
pub const LINEAGES: Verb = Verb {
    word: "lineages",
    params: &["workspace"],
    summary: "this workspace's config lineages, and the files each one holds",
    detail: "The policy branches a conversation is born on, each with its tip \
             commit and every file that commit holds. It is the listing a \
             config read then indexes into: pick a lineage, pick a path, read \
             the bytes. A workspace whose repository cannot be read says so \
             outright rather than answering no lineages at all.",
};

/// The `config` op's word, spelled once — the tag on a control and the
/// envelope it composes must be one string (`crate::ui::act`).
pub const CONFIG: &str = "config";

/// The destination's own field names, which are the wire's.
const TARGET_FILE: &str = "file";
const TARGET_NAME: &str = "name";
const TARGET_LINEAGE: &str = "lineage";
const TARGET_PATH: &str = "path";
const TARGET_ORIGIN: &str = "origin";

/// **The field that makes a `config` gesture a write.** It is beside `target`
/// and not inside it: the destination says WHERE and this says what lands.
const TEXT: &str = "text";

/// **Where a config read is addressed** — the five destinations, as the one
/// enum the pane picks and this module encodes.
///
/// A typed value rather than five builders, because the pane holds the choice
/// between frames and a `target` half-filled is a state neither end has a
/// reading for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Where {
    /// One workspace's own brazen `config.toml` (§9.1, §16.2).
    Brazen { workspace: String },
    /// litany's global models file — the engine's, naming no workspace.
    LitanyModels,
    /// One named litany workflow, on the same terms.
    LitanyWorkflow { name: String },
    /// yog's own cadence file, on the same terms.
    Cadence,
    /// One path on one of a workspace's config lineages (§9.3).
    Branch {
        workspace: String,
        lineage: String,
        path: String,
    },
}

impl Where {
    /// **The `target` object this destination is**, as the wire spells it.
    pub fn target(&self) -> Value {
        let mut map = Map::new();
        let mut put = |key: &str, value: &str| {
            map.insert(key.to_owned(), Value::String(value.to_owned()));
        };
        match self {
            Self::Brazen { workspace } => {
                put(TARGET_FILE, "brazen");
                put(envelope::WORKSPACE, workspace);
            }
            Self::LitanyModels => put(TARGET_FILE, "litany-models"),
            Self::LitanyWorkflow { name } => {
                put(TARGET_FILE, "litany-workflow");
                put(TARGET_NAME, name);
            }
            Self::Cadence => put(TARGET_FILE, "cadence"),
            Self::Branch {
                workspace,
                lineage,
                path,
            } => {
                put(TARGET_FILE, "branch");
                put(envelope::WORKSPACE, workspace);
                put(TARGET_LINEAGE, lineage);
                put(TARGET_PATH, path);
                // **The read carries the origin the WRITE would use**, because
                // upstream's decoder requires the field on every `branch`
                // destination and a read down a lineage is a read of its tip:
                // `advance` is that tip, where a fork or an orphan is a place
                // the write makes and no read can be about.
                put(TARGET_ORIGIN, "advance");
            }
        }
        Value::Object(map)
    }

    /// **What this box calls the destination**, which is what the pane paints
    /// on the control that picks it. One home, so the picker and the heading
    /// cannot spell one destination two ways.
    pub fn label(&self) -> String {
        match self {
            Self::Brazen { .. } => "brazen".to_owned(),
            Self::LitanyModels => "litany models".to_owned(),
            Self::LitanyWorkflow { name } => format!("workflow {name}"),
            Self::Cadence => "cadence".to_owned(),
            Self::Branch { lineage, path, .. } => format!("{lineage}: {path}"),
        }
    }

    /// **Whether this destination names a workspace** — and therefore whether
    /// §8.2 routes the gesture by it. The three that answer `false` name the
    /// engine itself, so the pane addresses them down a channel instead.
    pub fn addresses_a_workspace(&self) -> bool {
        matches!(self, Self::Brazen { .. } | Self::Branch { .. })
    }
}

/// **This workspace's config lineages**, asked of the wall `workspace` names.
pub fn lineages(workspace: String) -> Value {
    LINEAGES.built(vec![workspace])
}

/// **One config file's bytes, read** — the gesture with no `text`, which is the
/// whole of what makes it a read.
pub fn config(at: &Where) -> Value {
    addressed(at, None)
}

/// **One config file's bytes, written** — the same envelope carrying the whole
/// new text.
///
/// The WHOLE text and never a patch, because that is what upstream's pipeline
/// takes: `stage → validate → hash-guard → atomic rename` over the bytes the
/// gesture states (REMOTE §9.18, *"a typed edit is a seat composing that text
/// and applying it"*). A seat that sent a fragment would be asking the engine
/// to hold a draft.
pub fn write(at: &Where, text: String) -> Value {
    addressed(at, Some(text))
}

/// The one envelope both halves are, so a destination cannot be encoded two
/// ways: `text` present is the write and absent is the read, which is
/// upstream's own discriminator and this crate's only reading of it.
fn addressed(at: &Where, text: Option<String>) -> Value {
    let mut map = Map::new();
    map.insert(envelope::OP.to_owned(), Value::String(CONFIG.to_owned()));
    map.insert(envelope::TARGET.to_owned(), at.target());
    if let Some(text) = text {
        map.insert(TEXT.to_owned(), Value::String(text));
    }
    Value::Object(map)
}

#[cfg(test)]
mod tests;
