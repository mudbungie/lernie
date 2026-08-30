//! **What a model entry says** — the canonical content blocks, and the
//! provider's own token counters beside them.
//!
//! Split from [`super`] at the design-time budget, on the seam the vocabulary
//! itself draws: an entry is an *envelope* — a name, its bytes, which origin
//! wrote it — and this is what one origin's payload is made of. The two change
//! for different reasons, which is the test that a seam is real.
//!
//! **No provider vocabulary is pinned in either half.** A block kind this
//! build does not know keeps its word ([`Block::Unknown`], rung 3), and a
//! counter name is whatever the provider called it — so a counter the engine's
//! adapter starts reporting rides through with no edit here at all. The
//! alternative is a table this seat would have to be upgraded to keep, for a
//! surface whose whole content is somebody else's names.

use serde_json::{Map, Value};

use super::fields;

/// The provider's committed token counters, under the provider's own names.
///
/// Ordered rather than hashed, so the pane paints them in one order on every
/// box and two seats reading one entry never disagree about the layout.
/// **Empty is the general path**: an entry from before counters were sealed,
/// or a provider that reported none. A zero would be a lie, so absence stays
/// absence.
pub type Usage = std::collections::BTreeMap<String, u64>;

/// One content block of a model message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    /// Answer text.
    Text(String),
    /// Reasoning. It is **also** display text: a badge that never grows cannot
    /// tell a model thinking hard from a driver that has hung, so this becomes
    /// a row of its own rather than a spinner.
    Thinking(String),
    /// A tool call, painted as a chip. `input` is the summary the engine
    /// already made — never a second parse of the call's arguments, which
    /// would be this seat deciding what a tool call means.
    ToolUse {
        id: String,
        name: String,
        input: String,
    },
    /// A block kind this build does not know, verbatim (rung 3). It paints as
    /// its own word: a block silently dropped is a turn the operator reads as
    /// shorter than it was, which is the one failure a transcript must not
    /// have.
    Unknown(String),
}

impl Block {
    /// The word this block is written as — what an unstyled row labels it
    /// with.
    pub fn label(&self) -> String {
        match self {
            Self::Text(_) => TEXT.to_owned(),
            Self::Thinking(_) => THINKING.to_owned(),
            Self::ToolUse { .. } => TOOL_USE.to_owned(),
            Self::Unknown(word) => word.clone(),
        }
    }
}

const TEXT: &str = "text";
const THINKING: &str = "thinking";
const TOOL_USE: &str = "tool-use";

/// Read one block. The two text arms share a field name, which is the
/// engine's spelling and not a convenience: they are one thing said two ways,
/// and the kind is what says which.
pub(crate) fn block(v: &Value) -> Result<Block, String> {
    let o = v.as_object().ok_or("block: not an object")?;
    Ok(match fields::text(o, "kind")?.as_str() {
        TEXT => Block::Text(fields::text(o, TEXT)?),
        THINKING => Block::Thinking(fields::text(o, TEXT)?),
        TOOL_USE => Block::ToolUse {
            id: fields::text(o, "id")?,
            name: fields::text(o, "name")?,
            input: fields::text(o, "input")?,
        },
        other => Block::Unknown(other.to_owned()),
    })
}

/// Read the counters an entry carries.
///
/// The **object** is required and its emptiness is the reading: the encoder
/// writes `{}` rather than leaving the key out, because "reported nothing" is
/// what the bytes say and a missing key would make that indistinguishable from
/// an entry this codec failed to read. Every value must be a count — a counter
/// that is not a number is a shape failure and takes rung 1.
pub(crate) fn usage(o: &Map<String, Value>) -> Result<Usage, String> {
    o.get("usage")
        .and_then(Value::as_object)
        .ok_or_else(|| "missing or non-object field \"usage\"".to_owned())?
        .iter()
        .map(|(name, count)| {
            count
                .as_u64()
                .map(|n| (name.clone(), n))
                .ok_or_else(|| format!("usage {name:?}: not a count"))
        })
        .collect()
}

#[cfg(test)]
mod tests;
