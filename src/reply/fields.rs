//! **The reply codec's field readers** — rung 1 of [`super`]'s policy, in one
//! place so "shape refuses" is a property of the vocabulary rather than a
//! habit each decoder is trusted to keep.
//!
//! Every reader **names the field it refused on**. That is not politeness: a
//! seat's reader is the only party that can say which key of which answer was
//! wrong, and a bare "malformed reply" would send an operator to read the
//! engine's source.
//!
//! They are `pub(crate)`, not `pub`. Two take a reader function, which is a
//! trait bound, and a bound on a `pub` item forces monomorphisation onto every
//! consumer (`rules/no-pub-generic-bounds.yml`); the honest demotion is the
//! remedy the rule names, and nothing outside this crate reads a raw field.
//!
//! The key is always a parameter and the map never is a second borrow, so no
//! signature here needs a named lifetime: every reader takes borrows and hands
//! back an owned value, which is the house rule's *"borrow on the way in, own
//! on the way out"* with nothing to think about.

use serde_json::{Map, Value};

/// The key every listing hangs its elements off. One spelling, read by every
/// listing, because a second would be a second protocol.
const ROWS: &str = "rows";

/// A required string field.
pub(crate) fn text(obj: &Map<String, Value>, key: &str) -> Result<String, String> {
    obj.get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("missing or non-string field {key:?}"))
}

/// A required boolean field.
pub(crate) fn flag(obj: &Map<String, Value>, key: &str) -> Result<bool, String> {
    obj.get(key)
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("missing or non-boolean field {key:?}"))
}

/// A required signed-integer field — an age, which may be negative under clock
/// skew and is therefore not a count.
pub(crate) fn secs(obj: &Map<String, Value>, key: &str) -> Result<i64, String> {
    obj.get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("missing or non-integer field {key:?}"))
}

/// A required count — a rollup, a rank, an indent, a counter value.
///
/// **It keeps the wire's own width and is never narrowed to a `usize`.** The
/// narrowing would be free everywhere this crate is built and its failure arm
/// therefore unreachable, and an unreachable arm under a 100% floor is a line
/// no test can honestly cover — so the choice is a branch that lies about
/// being checked, or the width the answer was written in. A count is a count
/// at any width; the one place a narrowing is a real check is [`exit`], where
/// the type is the thing rather than a container for it.
pub(crate) fn count(obj: &Map<String, Value>, key: &str) -> Result<u64, String> {
    obj.get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("missing or non-integer field {key:?}"))
}

/// The captured run's exit status, narrowed the same way. Its own reader
/// rather than a call to [`secs`], because the narrowing is the strictness: a
/// status no `i32` holds is an engine saying something this seat has no way to
/// paint.
pub(crate) fn exit(obj: &Map<String, Value>) -> Result<i32, String> {
    let n = secs(obj, "exit")?;
    i32::try_from(n).map_err(|_| "field \"exit\" out of range".to_owned())
}

/// An **optional** string field: absent and `null` are both `None`, and a
/// value of the wrong type still refuses.
///
/// Absence is a reading here, never a malformed envelope — the reply surface
/// spells a fact's absence by leaving the key out precisely so a reader need
/// not tell "not stated" from "stated as empty". `None` and `Some("")` are two
/// different claims and this keeps them two.
pub(crate) fn opt_text(obj: &Map<String, Value>, key: &str) -> Result<Option<String>, String> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => text(obj, key).map(Some),
    }
}

/// An **optional** exit status: absent and `null` are both `None`, a value of
/// the wrong type refuses, and one no `i32` holds refuses too.
///
/// [`exit`]'s narrowing over [`opt_text`]'s absence, and it is a reader rather
/// than the two composed at the call site because the two failures have to name
/// one field between them: a sign-in that has not settled and one that settled
/// on a status this seat cannot paint are different claims (`super::login`).
pub(crate) fn opt_exit(obj: &Map<String, Value>, key: &str) -> Result<Option<i32>, String> {
    let Some(value) = obj.get(key).filter(|held| !held.is_null()) else {
        return Ok(None);
    };
    let n = value
        .as_i64()
        .ok_or_else(|| format!("missing or non-integer field {key:?}"))?;
    i32::try_from(n)
        .map(Some)
        .map_err(|_| format!("field {key:?} out of range"))
}

/// A boolean whose **absence is `false`**, and whose `null` is a refusal.
///
/// It is not [`opt_text`]'s shape one type over, and the difference is the
/// point: an optional string spells *not stated* by absence OR by `null`
/// because the reply surface uses both, while REMOTE §5.1's consent flag
/// *"absent reads false, rides only when true, and a mistyped value refuses at
/// the read"* — and upstream's own decoder refuses a `null` there. Two ends
/// disagreeing about what an absence is would be a consent read one way and
/// enforced the other.
pub(crate) fn absent_is_false(obj: &Map<String, Value>, key: &str) -> Result<bool, String> {
    match obj.get(key) {
        None => Ok(false),
        Some(_) => flag(obj, key),
    }
}

/// An **optional** signed integer: absent and `null` are both `None`, and a
/// value of the wrong type still refuses.
///
/// [`opt_text`]'s reading one type over, and it exists for the same reason: a
/// bound that is not stated and a bound of zero are two different claims about
/// what a control will accept (`super::config`).
pub(crate) fn opt_secs(obj: &Map<String, Value>, key: &str) -> Result<Option<i64>, String> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => secs(obj, key).map(Some),
    }
}

/// **An optional nested object**, on [`opt_text`]'s own terms: absent and
/// `null` are both `None`, and anything else is read strictly.
///
/// It lived beside its one caller (`super::queue`) while there was one, on the
/// rule that a reader with a single caller belongs there. `super::agent`
/// carries four of them, so it moved here — which is the rule spending itself
/// rather than being broken.
pub(crate) fn nested<T>(
    obj: &Map<String, Value>,
    key: &str,
    read: impl Fn(&Map<String, Value>) -> Result<T, String>,
) -> Result<Option<T>, String> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_object()
            .ok_or_else(|| format!("non-object field {key:?}"))
            .and_then(read)
            .map(Some),
    }
}

/// A required nested object, read strictly — the shape a reply always carries.
pub(crate) fn object<T>(
    obj: &Map<String, Value>,
    key: &str,
    read: impl Fn(&Map<String, Value>) -> Result<T, String>,
) -> Result<T, String> {
    obj.get(key)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("missing or non-object field {key:?}"))
        .and_then(read)
}

/// A listing's elements, each read by `read`.
///
/// One element that will not read fails the whole listing rather than
/// shortening it. A shorter list is a lie a window paints silently, which is
/// the one outcome [`super`]'s policy exists to exclude.
pub(crate) fn rows<T>(
    obj: &Map<String, Value>,
    read: impl Fn(&Value) -> Result<T, String>,
) -> Result<Vec<T>, String> {
    list(obj, ROWS, read)
}

/// The same, for an array under a key of its own.
pub(crate) fn list<T>(
    obj: &Map<String, Value>,
    key: &str,
    read: impl Fn(&Value) -> Result<T, String>,
) -> Result<Vec<T>, String> {
    obj.get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("missing or non-array field {key:?}"))?
        .iter()
        .map(read)
        .collect()
}

#[cfg(test)]
mod tests;
