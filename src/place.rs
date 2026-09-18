//! **Where the seat was pointed**, remembered between runs (yog's
//! `docs/REMOTE.md` §7; DESIGN §4.13).
//!
//! REMOTE §7 rules that per-seat UI state never crosses the boundary and is the
//! seat's own. This is the first of it that is durable, and where it goes is
//! the decision that mattered: under `paths::state_root` and never under
//! `paths::data_root`, because everything under the second is
//! operator-provisioned and irreplaceable by anything on this box, and a
//! regenerable subtree beside it would make a rebuild look like a revocation.
//! The whole of that argument is `crate::paths`'s module doc.
//!
//! # It may never become a way for the seat to fail to start
//!
//! [`read`] has exactly one answer for a file that is absent, unreadable,
//! truncated, or written by a build that spelled things differently: **no
//! place**, which is a window that opens on the roster — the same window a
//! first run gets. There is no refusal here and no repair path, because a
//! forgotten selection is a keypress and a startup error is an outage.
//!
//! That is also why the aim is not checked against anything. A wall that has
//! gone is a wall no channel resolves, and the standing set already declines to
//! ask about one (`crate::state::Standing::aimed`) — so a stale place is inert
//! by a rule that was already there, and validating it here would be a second
//! answer to a question already settled.
//!
//! # What it holds, and how it grows
//!
//! The wall the window was aimed at, the accordion beside it (DESIGN §4.39):
//! which engine is open, what each engine's last opening ranks as, and the
//! wall last aimed under each — and the edges the operator dragged, which are
//! the widths of the two list panes and how many rows the composer's field
//! stands at (§4.39 again, bl-46e5). **A JSON object rather than lines**, so
//! the next fact REMOTE §7 names — a scroll, a draft — is a key beside these
//! rather than a format. An unknown key is ignored and a missing
//! key is absence, which is the reply vocabulary's own rungs 3 and 4 applied to
//! this box's own file: a build that reads a file a newer build wrote loses
//! what it does not know and keeps what it does.
//!
//! **Why the accordion is here and not across the boundary**: §4.39's ruling,
//! and it is not §4.25's pin exception. A channel is a client-side fact, named
//! by this box and held by no other, so the order this box shows its channels
//! in can be nobody else's and there is no engine to assert it into.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::ui::{Aim, Dragged, Engines};

/// The file, under the state root. One name, so there is nothing to configure.
const FILE: &str = "place.json";
/// The aim, and the two words it is written as.
const AIM: &str = "aim";
const CHANNEL: &str = "channel";
const ADDRESS: &str = "address";
/// The accordion's three, beside the aim (DESIGN §4.39).
const OPEN: &str = "open";
const OPENED: &str = "opened";
const AIMED: &str = "aimed";
/// The dragged edges, one key each and flat: a width and a row count are
/// facts an operator set one at a time, not one object with a shape to keep.
const ROSTER_WIDTH: &str = "roster_width";
const CONVS_WIDTH: &str = "convs_width";
const COMPOSER_ROWS: &str = "composer_rows";

/// **Everything the seat remembers between runs.** One value, because it is
/// one file: read whole at boot and written whole once the event loop has
/// returned, so nothing here costs a frame anything.
///
/// **`Eq` is not derived** and cannot be: a dragged width is points, and a
/// float has no total equality. Nothing here needs one.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Place {
    /// The wall the window was aimed at, or nothing — which an operator who
    /// left the roster comes back to.
    pub aim: Option<Aim>,
    /// How the engines were arranged (`crate::ui::Engines`).
    pub engines: Engines,
    /// The edges the operator dragged (`crate::ui::Dragged`).
    pub dragged: Dragged,
}

impl Place {
    /// **The place a model IS**, as a projection and never as a stored copy:
    /// the aim and the accordion are the model's own fields, so there is
    /// nothing to keep in step and nothing that can disagree with the glass.
    pub fn of(model: &crate::ui::Model) -> Self {
        Self {
            aim: model.aim.clone(),
            engines: model.engines.clone(),
            dragged: model.dragged,
        }
    }
}

/// Where the place is kept under `root`.
pub fn at(root: &Path) -> PathBuf {
    root.join(FILE)
}

/// **Where the seat was pointed and how it was arranged**, or the empty place
/// — which is every way this can fail and the answer a first run gets.
pub fn read(root: &Path) -> Place {
    let held = std::fs::read_to_string(at(root))
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .unwrap_or(Value::Null);
    Place {
        aim: aimed(&held),
        engines: Engines {
            open: held.get(OPEN).and_then(Value::as_str).map(str::to_owned),
            opened: table(&held, OPENED, Value::as_u64),
            aimed: table(&held, AIMED, |held| held.as_str().map(str::to_owned)),
        },
        // **Every number is read through serde and never through a cast** —
        // `f64 as f32` is the one narrowing this crate's lint set denies with
        // no home for a suppression but the manifest (`crate::mark`'s own
        // reasoning). A key that is absent, null or some other shape entirely
        // is absence, which is rung 4 read on this box's own file — and here
        // absence is the policy's own answer rather than a missing number.
        dragged: Dragged {
            roster: width(&held, ROSTER_WIDTH),
            convs: width(&held, CONVS_WIDTH),
            rows: held
                .get(COMPOSER_ROWS)
                .and_then(|rows| serde_json::from_value::<u8>(rows.clone()).ok()),
        },
    }
}

/// One dragged width, in points, or absence.
fn width(held: &Value, key: &str) -> Option<f32> {
    serde_json::from_value(held.get(key)?.clone()).ok()
}

/// The aim, where the file carries a whole one. A half-written aim is no aim:
/// an address with no channel names nothing this seat can resolve.
fn aimed(held: &Value) -> Option<Aim> {
    let text = |key| held.get(AIM)?.get(key)?.as_str().map(str::to_owned);
    Some(Aim {
        channel: text(CHANNEL)?,
        address: text(ADDRESS)?,
    })
}

/// One name-keyed table, dropping every row this build cannot read — rung 3,
/// per row rather than per key, so one unreadable entry costs one entry.
fn table<T>(held: &Value, key: &str, read: impl Fn(&Value) -> Option<T>) -> BTreeMap<String, T> {
    held.get(key)
        .and_then(Value::as_object)
        .map(|rows| {
            rows.iter()
                .filter_map(|(name, held)| Some((name.clone(), read(held)?)))
                .collect()
        })
        .unwrap_or_default()
}

/// **Write the place down.** Aimed at nothing is a place too — an operator who
/// left the roster comes back to it — so this is called with whatever the last
/// frame held rather than only when there is something to say.
///
/// It answers a refusal rather than swallowing one. A read that fails has a
/// correct answer and this does not: the only alternative to saying so is
/// losing the operator's place in silence, and by the time this runs there is
/// no window left to paint it in.
pub fn write(root: &Path, place: &Place) -> Result<(), String> {
    let body = json!({
        AIM: place
            .aim
            .clone()
            .map(|aim| json!({ CHANNEL: aim.channel, ADDRESS: aim.address })),
        OPEN: place.engines.open,
        OPENED: place.engines.opened,
        AIMED: place.engines.aimed,
        ROSTER_WIDTH: place.dragged.roster,
        CONVS_WIDTH: place.dragged.convs,
        COMPOSER_ROWS: place.dragged.rows,
    });
    std::fs::create_dir_all(root).map_err(|e| format!("{}: {e}", root.display()))?;
    std::fs::write(at(root), body.to_string()).map_err(|e| format!("{}: {e}", at(root).display()))
}

#[cfg(test)]
mod tests;
