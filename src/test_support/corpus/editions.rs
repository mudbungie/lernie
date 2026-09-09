//! **Projecting and mutating a corpus frame** — the two edits the grows-only
//! contract is judged by (yog's `docs/REMOTE.md` §3.2).
//!
//! A vendored frame is what the NEWEST engine writes. The two things a client
//! must survive are an OLDER engine, which writes the same frame without the
//! fields stamped after its edition, and a NEWER one, which writes a word in a
//! vocabulary this build has never heard of. Neither can be captured, because
//! neither engine exists on the box running the suite — so both are made here,
//! out of the frame that does exist and the stamps beside it.
//!
//! The walkers are pure functions of a frame and a path, kept apart from the
//! replay that spends them (`crate::reply::tests::editions`) so each arm can be
//! exercised against a shape the vendored corpus does not yet have: at the cut
//! every path is at or below the floor, so a projection over real frames
//! deletes nothing and would leave the deletion itself untested.

use serde_json::Value;

use super::Signature;

/// Upstream's spelling for *every element of this array*.
const EVERY: &str = "[]";

/// The path segments of one signature entry, with its `:<type>` suffix already
/// stripped. A leading `/` makes the first segment empty, so it is dropped.
fn segments(path: &str) -> Vec<&str> {
    path.split('/').skip(1).collect()
}

/// **The field path and the JSON type upstream spells it with.** A signature
/// entry is `<path>:<type>`, and a nullable field is spelled twice — once per
/// type — so the type is part of the key rather than of the value.
pub(crate) fn spelled(entry: &str) -> (&str, &str) {
    entry.rsplit_once(':').unwrap_or((entry, ""))
}

/// **Every string-typed path in one signature**, which is where a vocabulary
/// word can be. The root and the reply's own `kind` are not among them: `kind`
/// is the discriminant and stays strict by name (DESIGN §4.9, rung 2).
pub(crate) fn words(signature: &Signature) -> Vec<String> {
    signature
        .keys()
        .map(|entry| spelled(entry))
        .filter(|(path, kind)| *kind == "string" && *path != "/kind" && !path.is_empty())
        .map(|(path, _)| path.to_owned())
        .collect()
}

/// **The frame an engine of edition `at` would have written**: every key
/// stamped later than `at` deleted, wherever it occurs.
pub(crate) fn project(frame: &Value, signature: &Signature, at: u32) -> Value {
    let mut projected = frame.clone();
    for (entry, stamp) in signature {
        if u32::try_from(*stamp).is_ok_and(|stamp| stamp <= at) {
            continue;
        }
        let (path, _) = spelled(entry);
        remove(&mut projected, &segments(path));
    }
    projected
}

/// Delete one path, following `[]` into every element on the way.
fn remove(value: &mut Value, segments: &[&str]) {
    let Some((leaf, above)) = segments.split_last() else {
        return;
    };
    walk(value, above, &mut |held| {
        if *leaf == EVERY {
            // The element spelling of a container: an engine that cannot write
            // the elements writes the empty array, which is the container's own
            // path saying the same thing one level down.
            if let Some(array) = held.as_array_mut() {
                array.clear();
            }
        } else if let Some(object) = held.as_object_mut() {
            object.remove(*leaf);
        }
    });
}

/// **One path set to `word`, wherever it occurs as a string.** `None` when the
/// frame carries no string there at all — a shape spells the union of what its
/// frames hold, so a path with nothing to mutate is the ordinary case and not
/// a fault.
pub(crate) fn mutate(frame: &Value, path: &str, word: &str) -> Option<Value> {
    let mut mutated = frame.clone();
    let segments = segments(path);
    let (leaf, above) = segments.split_last()?;
    let mut set = false;
    walk(&mut mutated, above, &mut |held| {
        if *leaf == EVERY {
            for element in held.as_array_mut().into_iter().flatten() {
                set |= replace(element, word);
            }
        } else if let Some(at) = held.as_object_mut().and_then(|o| o.get_mut(*leaf)) {
            set |= replace(at, word);
        }
    });
    set.then_some(mutated)
}

/// A string replaced in place, and anything else left alone.
fn replace(at: &mut Value, word: &str) -> bool {
    if !at.is_string() {
        return false;
    }
    *at = Value::String(word.to_owned());
    true
}

/// **Every value one path's segments reach, handed to `act`** — following `[]`
/// into each element, and reaching nothing where the path runs off this
/// particular frame, which is how an optional object absent here answers.
///
/// A visitor rather than a list of borrows: handing back `Vec<&mut Value>`
/// would put a named lifetime on the signature, which is the house rule's one
/// unconditional ban (`rules/no-named-lifetimes.yml`).
fn walk(value: &mut Value, segments: &[&str], act: &mut dyn FnMut(&mut Value)) {
    let Some((head, rest)) = segments.split_first() else {
        act(value);
        return;
    };
    if *head == EVERY {
        for element in value.as_array_mut().into_iter().flatten() {
            walk(element, rest, act);
        }
    } else if let Some(held) = value.get_mut(*head) {
        walk(held, rest, act);
    }
}

#[cfg(test)]
mod tests;
