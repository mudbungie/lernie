//! **A gesture that names no workspace, asked of every channel this box
//! holds** — the union, stamped with where each answer came from.
//!
//! The window has always done this: REMOTE §8.2's window half is *"the union
//! across those slices, local first and then the entries in leaf order, every
//! row carrying the channel it came from"*. The CLI's shorthand for the same
//! question did not, and the two surfaces disagreed on one box (bl-0d54) —
//! `lernie workspaces` answered this box's own engine and said nothing about
//! the rest, so a laptop that holds no engine of its own and is a client of a
//! server elsewhere was told nothing was provisioned, while its own window
//! painted the server's walls.
//!
//! **Which gestures fan is read off [`crate::verbs`]'s one table**, never
//! listed here: a verb with no `workspace` parameter has no way to name a
//! channel, so its subject is all of them ([`crate::verbs::Verb::addresses_a_workspace`]).
//!
//! **Nothing is rewritten on the way out.** §8.2's leaf↔host-name mapping is
//! spent at exactly one place, [`super::route`], and there is nothing to spend
//! here: the envelope names no workspace, so no entry's rename can apply to it.
//!
//! **A refusal is one channel's, never the set's** — the same rule the entries
//! directory has carried since §8.2. A channel that will not open or will not
//! answer says so in its own section while its neighbours answer; only a fan
//! that learned nothing at all is a failure, because only then was the question
//! unanswered.

use std::path::Path;

use serde_json::Value;

use super::{OWN, holds, lines};
use crate::channel::entries;
use crate::cli::Verdict;
use crate::envelope;

/// Ask `envelope` of every channel this box holds and answer with the union.
///
/// Each section is a channel's name as the roster spells it, then that
/// channel's own answer indented under it — the shape [`super::listing`]
/// already prints, because this is that listing *answered*.
pub fn fanned(data_root: &Path, envelope: &Value) -> Verdict {
    let asked: Vec<(String, Result<Vec<Value>, String>)> =
        std::iter::once((OWN.to_owned(), super::flat(data_root)))
            .chain(
                entries::read_dir(&entries::dir(data_root))
                    .into_iter()
                    .map(|held| (holds::label(&held.leaf, &held.workspace), held.open())),
            )
            .map(|(name, channel)| (name, channel.and_then(|open| open.ask(envelope))))
            .collect();
    let answered = asked.iter().any(|(_, said)| {
        said.as_ref()
            .is_ok_and(|stream| envelope::succeeded(stream))
    });
    let text = asked
        .iter()
        .map(|(name, said)| section(name, said))
        .collect::<Vec<String>>()
        .join("\n");
    Verdict::answered(text, answered)
}

/// One channel's section: its name, then what it said, indented under it.
///
/// The channel stamp is **this box's**, exactly as the window's is — no origin
/// crosses the wire and no reply grew a field the engine cannot fill.
fn section(name: &str, said: &Result<Vec<Value>, String>) -> String {
    let body = match said {
        Ok(stream) => lines(stream),
        Err(refusal) => refusal.clone(),
    };
    std::iter::once(name.to_owned())
        .chain(body.lines().map(|line| format!("    {line}")))
        .collect::<Vec<String>>()
        .join("\n")
}

#[cfg(test)]
mod tests;
