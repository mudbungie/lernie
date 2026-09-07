//! **The tool window** (yog's `docs/REMOTE.md` §5.5, PROTOCOL 15) — what is
//! running, where, and what it came back with, on the lane an operator has
//! open while it happens.
//!
//! # Two entries per call, and both are transitions
//!
//! The engine appends one entry as a call is dispatched (its name and its
//! bounded input) and another when the capture lands (its exit code), because
//! litany writes `input.json` and `output.json` at exactly those two moments —
//! the pair of file existences *is* the window opening and closing. So the
//! wire carries transitions and this end holds **calls**: [`fold`] merges the
//! two halves onto one [`Window`] keyed by [`Window::tool_use`], which is what
//! lets the closing entry restate neither the name nor the input.
//!
//! # Presence is the status, and there is no third reading
//!
//! [`Window::exit_code`] absent is a call **in flight**; present is one whose
//! capture landed. REMOTE §5.5 is explicit that there is no arm for *complete,
//! status unknown*, which is what makes the two readings impossible to
//! disagree about.
//!
//! # Where it ran is in the name, and this end must not take it apart
//!
//! REMOTE §5.1 presents a loaded remote tool as `<client>_<tool>`, always and
//! never only when ambiguous, so `box2_Bash` says the box as well as the tool.
//! The engine does not split that composition back apart and neither does
//! this: the registry answers *where a call would route now*, which is a
//! different question from where this one ran.

use serde_json::{Map, Value};

use super::super::fields;

/// The key the window rides under, beside the fold rather than inside it.
pub(crate) const TOOLS: &str = "tools";

/// The three fields an entry can carry beside its id.
const TOOL: &str = "tool";
const INPUT: &str = "input";
const EXIT_CODE: &str = "exit_code";

/// **One tool call, as much of it as has been said.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    /// The call's id — the only field both transitions carry, and therefore
    /// the one thing that pairs them.
    pub tool_use: String,
    /// `<client>_<tool>`: what ran and, for a routed call, the machine it ran
    /// on. Absent on a frame carrying only the closing half of a call whose
    /// opening landed in an earlier frame of the same read.
    pub tool: Option<String>,
    /// The call's input, already bounded and flattened to one line by the
    /// engine (REMOTE §5.5) — this lane exists to be read *while* it streams,
    /// and a whole file body is an ordinary write argument.
    pub input: Option<String>,
    /// **The status, by its presence.** Absent is a call still in flight.
    pub exit_code: Option<i32>,
}

/// The window a frame carries. **Required, empty list included** (REMOTE
/// §5.5): absent would make *this build has no tool window* and *nothing ran
/// since the last frame* one shape, which is the reassuring answer on exactly
/// the build that cannot tell.
pub(crate) fn windows(obj: &Map<String, Value>) -> Result<Vec<Window>, String> {
    fields::list(obj, TOOLS, entry)
}

/// One transition.
fn entry(value: &Value) -> Result<Window, String> {
    let obj = value.as_object().ok_or("tool window: not a JSON object")?;
    Ok(Window {
        tool_use: fields::text(obj, "tool_use")?,
        tool: fields::opt_text(obj, TOOL)?,
        input: fields::opt_text(obj, INPUT)?,
        exit_code: fields::opt_exit(obj, EXIT_CODE)?,
    })
}

/// **Merge one transition into the calls held so far**, keyed by the id both
/// halves carry — the whole of how a closing entry that restates nothing is
/// still read as the end of a named call.
pub(crate) fn fold(held: &mut Vec<Window>, entry: Window) {
    match held.iter_mut().find(|call| call.tool_use == entry.tool_use) {
        // Every field is filled where the later entry has one and left alone
        // where it does not, so a transition is additive whichever half it is
        // and neither half can blank the other's facts.
        Some(call) => {
            call.tool = entry.tool.or_else(|| call.tool.take());
            call.input = entry.input.or_else(|| call.input.take());
            call.exit_code = entry.exit_code.or(call.exit_code);
        }
        None => held.push(entry),
    }
}

#[cfg(test)]
mod tests;
