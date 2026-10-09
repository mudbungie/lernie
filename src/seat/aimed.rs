//! **A world-level ACT, down the one channel the operator named** (bl-1bb9;
//! DESIGN §4.41).
//!
//! `price` and `ceiling` name no workspace — the table and the bound are world
//! facts, one world per engine (yog DESIGN §4.1) — so there is nothing for
//! §8.2's mapping to resolve, and the channel is named outright, exactly as
//! the window's control names the section it fired from
//! ([`crate::offframe::down`]). The name is read by [`super::dial`], the one
//! place a channel's name opens it; this module adds no resolver of its own.
//!
//! **A write is not a read, and only a read fans.** An operator typing one
//! number means one engine, so a box holding several channels refuses an act
//! that names none, naming what it holds — and a box holding exactly one has
//! one channel to write, which is the ordinary case and needs no name.

use std::path::Path;

use serde_json::Value;

use super::{dial, fan, said};
use crate::channel::{Channel, Reach};
use crate::cli::{ON, Verdict};
use crate::envelope;
use crate::render::Form;

/// Ask `envelope` of the channel `on` names, or of the one channel this box
/// holds where it names none.
pub fn aimed(data_root: &Path, on: Option<&str>, envelope: &Value, form: Form) -> Verdict {
    let channel = on.map_or_else(|| sole(data_root, envelope), |name| dial(data_root, name));
    match channel
        .map_err(Reach::Unsent)
        .and_then(|open| open.ask(envelope))
    {
        Ok(stream) => Verdict::answered(said(&stream, form), envelope::succeeded(&stream)),
        Err(reach) => Verdict::failed(reach.said()),
    }
}

/// **The one channel this box holds**, or the refusal naming every one it
/// does — the same enumeration the fan asks ([`fan::held`]).
fn sole(data_root: &Path, envelope: &Value) -> Result<Channel, String> {
    match <[_; 1]>::try_from(fan::held(data_root)) {
        Ok([(_, channel)]) => channel,
        Err(held) => Err(format!(
            "this box holds {} channels and `{}` writes one engine's world, so \
             name the one with `{ON} <channel>`: {}",
            held.len(),
            envelope::op(envelope),
            held.into_iter()
                .map(|(name, _)| name)
                .collect::<Vec<String>>()
                .join(", ")
        )),
    }
}

#[cfg(test)]
mod tests;
