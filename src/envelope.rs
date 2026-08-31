//! **The gesture envelope, from the seat's side** (yog's `docs/REMOTE.md` §3,
//! §8.2; DESIGN §4.1).
//!
//! REMOTE §3 is emphatic that *"the wire is a transport for the boundary, not a
//! vocabulary"*: what crosses is the same JSON envelope the engine's own
//! `gestures/` inbox carries, `op` the discriminant and every parameter a named
//! field. **So the seat carries the envelope; it does not own it.** This module
//! is deliberately the whole of what a seat has to understand about one, and
//! it is three things:
//!
//! - that it **is** an object with an `op` — enough to refuse a typo at the
//!   seat rather than spend a connection on it;
//! - **which workspace it names**, because that is what decides the channel it
//!   goes down (§8.2);
//! - whether the last reply frame said **ok**, because that is the exit code.
//!
//! Nothing else is read and nothing else is written. A gesture the engine has
//! learned and this build has not crosses unchanged and is answered normally,
//! which is REMOTE §3's own rule that a new verb is not a protocol bump: an
//! `op` the far end does not know refuses in band, naming it, and that is the
//! boundary correcting itself rather than two protocols meeting.
//!
//! **The typed ASK vocabulary is NOT here and its absence is the design.** yog
//! holds `Action`/`Query` and their codec because the engine adjudicates them.
//! Reimplementing that side to *route* a gesture would be a second table over
//! thirty-odd variants whose only job is to answer a question one field already
//! answers — and the arm that drifted would send a client's own leaf to a host
//! that never heard of it.
//!
//! [`crate::verbs`] is not that table and does not become one. It is a
//! **serialization**: a word and its parameters, built into the envelope below
//! and then routed by [`workspace`] exactly as a hand-written one is. Nothing
//! there is consulted about where a gesture goes, and its rows carry no
//! knowledge of what an op means — which is why an op it does not name still
//! crosses, through `ask`, unchanged.
//!
//! The **reply** half is [`crate::reply`], and it is the other side of the same
//! ruling: it decodes what the window paints and nothing else.
//!
//! **One table, not two** (§8.2's mapping is spent in both directions). The
//! read answers *through* the write: [`slot_mut`] is the whole of where an
//! envelope names its workspace, and [`workspace`] and [`with_workspace`] are
//! both spelled in terms of it. Two matches over one fact is the arm that
//! drifts.

use serde_json::{Map, Value};

/// The envelope's discriminant. Read only to refuse something that is not a
/// gesture at all; never interpreted.
pub const OP: &str = "op";
/// The field naming the workspace a gesture is aimed at.
pub const WORKSPACE: &str = "workspace";
/// The nested body two of the start family's envelopes carry their workspace
/// inside (`prompt`, `fan`) — one of the two places the name is a level down.
pub const PREPARED: &str = "prepared";
/// The destination the config family carries its workspace inside — the other
/// place the name is a level down, and the wall a config act edits.
pub const TARGET: &str = "target";
/// The reply envelope's verdict field.
pub const OK: &str = "ok";

/// Read one envelope off text. **Strict, for REMOTE §3's reason**: a gesture is
/// an instruction, not an observation, so a body that is not JSON, is not an
/// object, or names no `op` refuses here rather than being sent and refused
/// there. Guessing at a malformed instruction is worse than refusing one.
pub fn parse(text: &str) -> Result<Value, String> {
    let value: Value = serde_json::from_str(text).map_err(|e| format!("not JSON: {e}"))?;
    let obj = value
        .as_object()
        .ok_or_else(|| "not a gesture envelope: a gesture is a JSON object".to_owned())?;
    match obj.get(OP) {
        Some(Value::String(_)) => Ok(value),
        Some(_) => Err(format!("field {OP:?} is not a string")),
        None => Err(format!(
            "not a gesture envelope: missing field {OP:?}, the discriminant \
             every gesture carries"
        )),
    }
}

/// The workspace this envelope names, or `None` when it names none — the
/// roster, the board, the trail and every gesture whose subject is the whole
/// world.
pub fn workspace(envelope: &Value) -> Option<String> {
    let mut named = envelope.clone();
    slot_mut(&mut named).map(std::mem::take)
}

/// This envelope with the workspace it names replaced by `name`.
///
/// An envelope naming no workspace comes back byte for byte: the general path
/// with nothing to rewrite, not a case of its own.
pub fn with_workspace(envelope: &Value, name: &str) -> Value {
    let mut written = envelope.clone();
    if let Some(slot) = slot_mut(&mut written) {
        name.clone_into(slot);
    }
    written
}

/// **The one table**: where an envelope names its workspace, borrowed so the
/// read and the rewrite cannot disagree about which envelopes have one.
///
/// Three shapes, because yog's own typed table has three: the field is top
/// level on every gesture that addresses a workspace directly, one level down
/// inside `prepared` on the two that carry a prepared start (`prompt`, `fan`),
/// and one level down inside `target` on the config family, whose destination
/// *is* its address. The two nested shapes are load-bearing rather than
/// oddities. The name inside a prepared body is handed straight back out as the
/// next act's address, so a prepared left in the host's spelling routes its own
/// follow-up to a name no entry claims. The name inside a config destination is
/// the wall whose file the act edits, so a config act aimed at an entry under a
/// §8.2 rename would otherwise resolve to no entry at all, fall through to this
/// box's own engine, and write the wrong wall's file — silently (bl-4a36, and
/// yog's twin bl-523f fixing the same row in the typed table).
///
/// **What is still deliberately NOT a slot**: a `workspace` nested anywhere
/// else. Three holders, in the order they are read, and a top-level name always
/// wins — no envelope in the vocabulary carries two, and reading the outer one
/// first is what keeps that true if one ever does.
fn slot_mut(envelope: &mut Value) -> Option<&mut String> {
    let obj = envelope.as_object_mut()?;
    let holder = if obj.contains_key(WORKSPACE) {
        obj
    } else if obj.contains_key(PREPARED) {
        obj.get_mut(PREPARED)?.as_object_mut()?
    } else {
        obj.get_mut(TARGET)?.as_object_mut()?
    };
    named(holder)
}

/// The [`WORKSPACE`] field of an object, borrowed as the name it is.
///
/// The key is not a parameter, and that is not thrift: two borrowed arguments
/// would need a named lifetime to say which one the answer comes out of, and a
/// named lifetime on a signature leaks internal storage into the interface
/// (`rules/no-named-lifetimes.yml`). There is exactly one key here, so there is
/// no question to answer.
///
/// A field that is present and is not a string is not a slot: the seat rewrites
/// what it can read as a name and leaves everything else exactly as the
/// operator wrote it.
fn named(obj: &mut Map<String, Value>) -> Option<&mut String> {
    match obj.get_mut(WORKSPACE) {
        Some(Value::String(s)) => Some(s),
        _ => None,
    }
}

/// **Whether a reply stream succeeded**, which is the seat's exit code.
///
/// The LAST frame decides: an ordinary answer is one frame, and a follow-class
/// read's last frame is its newest state. An empty stream is an engine that
/// terminated without answering, and that is not ok. Neither is a frame that
/// carries no `ok` — a seat does not read a missing verdict as a good one.
pub fn succeeded(stream: &[Value]) -> bool {
    stream
        .last()
        .and_then(|reply| reply.get(OK))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests;
