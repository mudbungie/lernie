//! **One config file, answered as the typed thing it is** (yog's
//! `docs/REMOTE.md` §9.18; PROTOCOL 13) — the bytes, and the schema applied to
//! those very bytes.
//!
//! # One answer, both views, and never two reads
//!
//! Upstream states the reasoning and it is why there is no second op to ask:
//! *"The reply now carries the destination's `text` and, beside it, that file's
//! schema applied to those very bytes — never a second read, which could be of
//! a different state."* So a seat that joined a schema against a separate byte
//! read would be painting two moments as one.
//!
//! # A file with no schema answers an EMPTY array, not an absent key
//!
//! §9.5's justified raw-text destinations — brazen's `config.toml`, a
//! workflow, a config commit's prose paths — are *"the general path with empty
//! input rather than a branch"*, and a pane holding no setting is already
//! showing the raw editor. So [`Config::settings`] is required and may be
//! empty, and its emptiness is a reading rather than an absence.
//!
//! # `fault` is absent, never null, and the seat never composes one
//!
//! It is the engine's judgement of that value in words — the same call §9.4's
//! pick gate makes — so a seat that derived its own would be a second authority
//! on the far side of a boundary. Absent is *nothing is wrong with this value*;
//! it is not *nobody looked*, because an unanswerable table faults nothing.
//!
//! # The bounds ride the control, and this seat carries them unread today
//!
//! `{"kind":"number","min":…,"max":…}` is one shape rather than four optional
//! siblings, because judging at input cannot be done without the range. This
//! build paints the values and does not yet edit them, so the bounds are
//! decoded and stated rather than enforced — [`Control::says`] is that
//! sentence, and the editor that judges against them is bl-4bb1's.

use serde_json::{Map, Value};

use super::fields;

/// The kind token this reading answers to.
pub(crate) const KIND: &str = "config";

/// One config file: its whole text, and the settings its schema found in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// The file's bytes, verbatim. Empty for a destination that does not exist
    /// yet, which upstream answers rather than refusing.
    pub text: String,
    /// Every setting the file declares, flat and each naming its own entry.
    /// Empty where the destination has no schema at all.
    pub settings: Vec<Setting>,
}

/// One typed setting the schema found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Setting {
    /// **Which declaration it belongs to** — a role in `providers.yaml`, a
    /// model id in `models.yaml`, a section in `cadence.yaml`. The rows group
    /// by it, and the grouping is this end's: upstream keeps no second shape
    /// for it.
    pub entry: String,
    /// The setting's own name within that entry.
    pub name: String,
    /// What the file currently says, as a string whatever the control is.
    pub value: String,
    /// What it is for, in the engine's words.
    pub help: String,
    /// **The engine's judgement of this value**, where it has one. Absent is
    /// *nothing is wrong with it*.
    pub fault: Option<String>,
    /// What kind of control the value takes, and the range it is legal in.
    pub control: Control,
}

/// The control a setting takes, and the bounds it is legal in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Control {
    /// The control's word, carried verbatim — [`super`]'s rung 3: a kind this
    /// build has never seen paints as itself rather than as a neighbour.
    pub kind: String,
    /// The lowest legal value, where the kind carries one.
    pub min: Option<i64>,
    /// The highest, on the same terms.
    pub max: Option<i64>,
}

impl Control {
    /// **What the control says about itself in one clause** — its kind, and
    /// the range where it has one.
    pub fn says(&self) -> String {
        match (self.min, self.max) {
            (Some(min), Some(max)) => format!("{} {min}–{max}", self.kind),
            _ => self.kind.clone(),
        }
    }
}

/// The whole answer.
pub(crate) fn config(obj: &Map<String, Value>) -> Result<Config, String> {
    Ok(Config {
        text: fields::text(obj, "text")?,
        settings: fields::list(obj, "settings", setting)?,
    })
}

/// One setting row.
fn setting(value: &Value) -> Result<Setting, String> {
    let obj = value.as_object().ok_or("setting: not a JSON object")?;
    Ok(Setting {
        entry: fields::text(obj, "entry")?,
        name: fields::text(obj, "name")?,
        value: fields::text(obj, "value")?,
        help: fields::text(obj, "help")?,
        fault: fields::opt_text(obj, "fault")?,
        control: control(obj.get("control").ok_or("setting: missing control")?)?,
    })
}

/// The control object beside a setting.
fn control(value: &Value) -> Result<Control, String> {
    let obj = value.as_object().ok_or("control: not a JSON object")?;
    Ok(Control {
        kind: fields::text(obj, "kind")?,
        min: fields::opt_secs(obj, "min")?,
        max: fields::opt_secs(obj, "max")?,
    })
}

#[cfg(test)]
mod tests;
