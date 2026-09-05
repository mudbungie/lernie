//! **One sign-in run, as the engine is streaming it** (yog's `docs/REMOTE.md`
//! §8.3; PROTOCOL 13).
//!
//! The act and the read answer one kind, which is upstream's own decision and
//! not a coincidence: *"an act's receipt and a lane's first frame are the same
//! value at the same moment"*. So `login` and `login-tail` are one reading here
//! too, and the pane paints the same rows whichever gesture produced them.
//!
//! # A frame is an APPEND, exactly as the live tail's is
//!
//! `Query::LoginTail` hands over what the run has said **since the last frame**
//! — the engine's lane advances a per-read cursor — so the first frame of any
//! read is the whole buffer and a seat that lost its connection re-asks and is
//! whole again. [`Signin::absorb`] is that fold, and it is
//! [`super::stream::Stream::absorb`]'s rule one noun over: lines accrete in
//! order, and the settled facts of the last frame win.
//!
//! # Both streams are here, tagged, because bz writes the flow to stderr
//!
//! The authorize URL, a device code, and a failure's exact reason and remedy
//! are all stderr. A reader that kept only stdout paints a blank pane and
//! leaves the operator with nothing but a command to retype (yog bl-b4e5).
//!
//! # The two settled facts are absent while it runs, and absence is a reading
//!
//! `outcome` is the terminal exit status, and `fallback` is set **only** on a
//! non-zero one — the command to run by hand, in the workspace-bound spelling
//! the engine composes because it is the end that knows the wall. Neither is
//! `null` on the wire while the run is live, so neither is a gap here: a run
//! with no outcome is a run that has not finished.
//!
//! # A pair nobody has signed in to is one empty frame, never silence
//!
//! Upstream opens the lane on a `LoginView::default()`, which reads here as a
//! [`Signin`] with no lines and no outcome. That is *nobody has signed in to
//! this row in this workspace* and the pane says so in words — the honesty rule
//! REMOTE §10 states for every held read.

use serde_json::{Map, Value};

use super::fields;

/// The kind token both the act's receipt and the lane's frames answer to.
pub(crate) const KIND: &str = "login";

/// One line the sign-in printed, and which stream it came down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    /// The line, verbatim. Nothing here trims, wraps or normalises it.
    pub text: String,
    /// Whether it came down stderr, which is where bz writes the human-facing
    /// flow. It is painted as a tone rather than hidden: an authorize URL and
    /// a failure's remedy arrive on the same stream.
    pub err: bool,
}

/// What one read of a sign-in run says, folded.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Signin {
    /// Everything the run has printed so far, in order.
    pub lines: Vec<Line>,
    /// The terminal exit status once it settles, or `None` while it runs.
    pub outcome: Option<i32>,
    /// The command to run by hand, set only on a non-zero exit.
    pub fallback: Option<String>,
}

impl Signin {
    /// **Absorb the frame that landed after this one** — the §8.3 lane's
    /// reassembly, and [`super::stream::Stream::absorb`]'s contract one noun
    /// over: `fold(a).absorb(fold(b)) == fold(a ++ b)` on any frame boundary.
    ///
    /// The lines accrete because each frame carries only what the run said
    /// since the last. The two settled facts **replace**, because a run settles
    /// once and the frame that carries the exit is the last one: a later frame
    /// that said nothing about them would otherwise unsettle a run this end has
    /// already been told about.
    pub fn absorb(&mut self, mut later: Self) {
        self.lines.append(&mut later.lines);
        self.outcome = later.outcome.or(self.outcome.take());
        self.fallback = later.fallback.or(self.fallback.take());
    }

    /// **Whether the run has settled.** Derived from the outcome rather than
    /// held beside it — one fact, one home.
    pub fn settled(&self) -> bool {
        self.outcome.is_some()
    }
}

/// The key the lines ride under. Not [`super::fields`]'s `rows`: this listing
/// is one run's output rather than a table, and upstream spells it as what it
/// is.
const LINES: &str = "lines";

/// Read one line, strictly.
fn line(value: &Value) -> Result<Line, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("sign-in line: not an object")?;
    Ok(Line {
        text: fields::text(obj, "text")?,
        err: fields::flag(obj, "err")?,
    })
}

/// Read the reply: the lines, and the two facts a settled run carries.
pub(crate) fn signin(obj: &Map<String, Value>) -> Result<Signin, String> {
    Ok(Signin {
        lines: fields::list(obj, LINES, line)?,
        outcome: fields::opt_exit(obj, "outcome")?,
        fallback: fields::opt_text(obj, "fallback")?,
    })
}

#[cfg(test)]
mod tests;
