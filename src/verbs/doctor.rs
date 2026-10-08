//! **The doctor** — whether this box is wired, asked as one read (yog's
//! `docs/REMOTE.md`, *"And one read asks all of it at once"*, bl-28f4; bl-9bbb
//! here).
//!
//! # A door, because its one parameter is optional
//!
//! The workspace is optional, *"which no other workspace-addressed read is"*,
//! for the reason the gesture exists: the box it is for may hold no workspace
//! at all. A row of [`super`]'s table names its parameters and a parameter is
//! required by being one, so this is a typed door beside [`super::trail`]'s,
//! and argv spells it in [`super::doors`] — `lernie doctor [<workspace>]`.
//!
//! # Where it goes is read off the envelope, as everywhere
//!
//! Bare, it names no workspace and so fans: every channel this box holds is
//! asked about its own wiring, and the answer is their union. Naming one, it
//! goes down that wall's channel alone and adds the wall's two checks
//! (`crate::envelope::workspace`; DESIGN §4.21). **Absent is absent**: the
//! bare form states no field, never a `null`.

use serde_json::{Map, Value};

use crate::envelope;

/// The word this door spells, and the envelope's `op`. One fact.
pub const DOCTOR: &str = "doctor";

/// **The doctor**, about the box alone or about one wall on it as well.
pub fn doctor(workspace: Option<String>) -> Value {
    let mut map = Map::new();
    map.insert(envelope::OP.to_owned(), Value::String(DOCTOR.to_owned()));
    if let Some(named) = workspace {
        map.insert(envelope::WORKSPACE.to_owned(), Value::String(named));
    }
    Value::Object(map)
}
