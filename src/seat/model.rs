//! **Assigning a model to a role, against the list this seat can already
//! fetch** (bl-1e5a).
//!
//! `model` accepted a model id the same seat could prove does not exist,
//! answered `ok`, wrote it into the wall's `providers.yaml`, and the workspace
//! was then broken until somebody read a step record: the next deposit died,
//! and the failure reached the operator truncated, on a row, a turn later.
//!
//! **The documented rule is the ENGINE's and it is the right rule** — *a model
//! id is validated by the wire at the first live model call* — because an
//! engine that held a second opinion about a provider's catalogue would be a
//! second authority on somebody else's table. It is not an argument for the
//! SEAT staying silent when it has just been handed a word it can check
//! against a list it already knows how to fetch: `models` is a row in the same
//! table, one round trip, off the same channel.
//!
//! # A warning, and never a refusal
//!
//! The seat is not that second authority either. A provider's list can move
//! under this seat between the read and the write, a cache can be stale, and a
//! row that cannot be asked answers nothing at all — so a mismatch is said and
//! the assignment goes through. What is bought is that the operator learns it
//! now, in full, rather than a turn later in twelve truncated words.
//!
//! **And the whole offered list is named, not a guess at the nearest.** It is
//! the same round trip either way, it is exactly what `lernie models` would
//! have printed, and a similarity score is a mechanism with no input: the
//! operator knows which one they meant the moment they see the row.
//!
//! # What silence means here
//!
//! A `models` read that refuses, fails to reach, or answers a kind this build
//! cannot read says nothing and the assignment proceeds. The check is a
//! courtesy this seat can extend when the answer is in hand, and turning its
//! absence into a refusal would make an unreachable provider row a reason not
//! to write a file the engine is perfectly willing to write.

use std::path::Path;

use crate::cli::Verdict;
use crate::render::Form;
use crate::reply::{Read, Reply, read};

/// **Give a role this model**, having first asked the provider row what it
/// offers. `warn` takes the sentence a mismatch earns; the verdict is the
/// assignment's own.
pub fn model(
    data_root: &Path,
    workspace: &str,
    role: &str,
    provider: &str,
    model: &str,
    form: Form,
    warn: &mut dyn FnMut(&str),
) -> Verdict {
    if let Some(offered) = offered(data_root, workspace, provider)
        && !offered.iter().any(|known| known == model)
    {
        warn(&unlisted(provider, model, &offered));
    }
    super::ask(
        data_root,
        &crate::verbs::model(
            workspace.to_owned(),
            role.to_owned(),
            provider.to_owned(),
            model.to_owned(),
        ),
        form,
    )
}

/// What the provider row is offering, when it answered a listing at all.
///
/// **Every other outcome is `None` and says nothing** — see the module doc.
/// An empty listing is `None` too: a row offering nothing has told this seat
/// that it cannot say what exists, not that the id is wrong.
fn offered(data_root: &Path, workspace: &str, provider: &str) -> Option<Vec<String>> {
    let asking = crate::verbs::models(workspace.to_owned(), provider.to_owned());
    let answered = super::sent(data_root, &asking).ok()?;
    match answered.last().map(read) {
        Some(Read::Answer(Reply::Models(rows))) if !rows.is_empty() => Some(rows),
        _ => None,
    }
}

/// The sentence a model id nobody offers earns: what was asked for, that it is
/// going through anyway, and the whole row it is not in.
fn unlisted(provider: &str, model: &str, offered: &[String]) -> String {
    format!(
        "lernie: {provider:?} does not offer {model:?} — assigning it anyway, because the \
         list belongs to the provider and the engine validates at the first live call. It \
         offers: {}",
        offered.join(", ")
    )
}

#[cfg(test)]
mod tests;
