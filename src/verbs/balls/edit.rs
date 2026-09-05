//! **The two authoring acts** — filing a ball and amending one (bl-f7ae; yog's
//! `docs/REMOTE.md` §9.7).
//!
//! Its own file beside [`super`]'s rows on the seam upstream already cuts:
//! yog's `actions/verbs/balls/edit.rs` holds exactly these two payloads and
//! `verb.rs` holds the rest of the family, because *"one vocabulary, so a fact
//! balls learns is added in one place"*. The seam is the same one this side,
//! and it is the design-time budget's answer as well (DESIGN §5): three of the
//! family's acts are rows and two can never be.
//!
//! # Neither can be a row, and the reason is absence
//!
//! [`super::super`]'s table is *a word and its parameters, all of them named
//! strings*, built by one builder that writes every parameter it names. Both
//! verbs here carry text that may be **absent**: `create`'s body, and each of
//! `update`'s three. Absence is a value on this wire — yog's decoder reads
//! `body` with `opt_str_of` and an empty string is a different claim from a
//! missing key — so a row would have to send `""` where the operator wrote
//! nothing, which asks upstream to blank a field nobody touched.
//!
//! So they are typed doors with no row, on `effort`'s terms (a level that is a
//! string **or null**) and `fork`'s (a list). Same rule, third application.
//!
//! # `update` sends only the fields that were typed, which is the whole point
//!
//! yog refuses an `update` that changes nothing — *"at least one field is
//! required, or the line asked for nothing"* — and it appends `note` to the
//! journal rather than replacing it. Both readings fall out of spelling
//! absence as absence: a control that sent three empty strings would clear a
//! title and a body to amend a journal.

use serde_json::{Map, Value};

use super::{ID, NAME, PROJECT};
use crate::envelope;

/// The word a filing's control names, which has no row to read it off.
pub const CREATE: &str = "create";
/// The word an amendment's control names.
pub const UPDATE: &str = "update";

/// The text fields the two doors carry.
const TITLE: &str = "title";
const BODY: &str = "body";
const NOTE: &str = "note";

/// **File a ball in `project`, stamped `--as name`.**
///
/// The title is required and the body is not: yog's own grammar is *"the title
/// is the words before any flag; `--body` carries the rest of the
/// description"*, and a ball filed with no description is the ordinary case
/// rather than a short one.
pub fn create(project: String, name: String, title: String, body: Option<String>) -> Value {
    authored(
        CREATE,
        vec![(PROJECT, project), (NAME, name), (TITLE, title)],
        vec![(BODY, body)],
    )
}

/// **Amend a ball's title, body, or its journal.**
///
/// Every one of the three is optional and each is a different act: the first
/// two replace, and `note` appends. A `None` is a field the operator did not
/// touch, and it is left out of the envelope entirely.
pub fn update(
    project: String,
    id: String,
    name: String,
    title: Option<String>,
    body: Option<String>,
    note: Option<String>,
) -> Value {
    authored(
        UPDATE,
        vec![(PROJECT, project), (ID, id), (NAME, name)],
        vec![(TITLE, title), (BODY, body), (NOTE, note)],
    )
}

/// **The one builder both doors arrive at**, so an authoring gesture has one
/// spelling however it was composed — [`super::super::Verb::built`]'s own rule,
/// kept where the table's builder cannot go.
///
/// `said` is written always and `absent_is_a_value` only where there is
/// something to write, which is the whole of what makes these doors rather than
/// rows.
fn authored(
    op: &str,
    said: Vec<(&str, String)>,
    absent_is_a_value: Vec<(&str, Option<String>)>,
) -> Value {
    let mut map = Map::new();
    map.insert(envelope::OP.to_owned(), Value::String(op.to_owned()));
    for (key, value) in said {
        map.insert(key.to_owned(), Value::String(value));
    }
    for (key, value) in absent_is_a_value
        .into_iter()
        .filter_map(|(k, v)| Some((k, v?)))
    {
        map.insert(key.to_owned(), Value::String(value));
    }
    Value::Object(map)
}

#[cfg(test)]
mod tests;
