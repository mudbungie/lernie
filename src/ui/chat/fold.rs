//! **What a machine's answer hides when it is folded** (bl-90d0).
//!
//! litany bounds a tool stream at 16 KiB head plus 16 KiB tail before it ever
//! reaches a transcript, so up to 32 KiB per call arrives here — and the pane
//! painted all of it. One `bash` running `ls -la && git log` filled the whole
//! pane and continued below it; one `bl --help` was six screens; a `find` over
//! a home tree arrived essentially whole. The call above each of them is one
//! compact line, which is what makes the asymmetry sharp: the question is a
//! line and its answer is six screens.
//!
//! **This is a projection, not a control.** The fold is a value computed from
//! the content — a few lines, and a sentence saying what is not being shown —
//! so what the pane paints folded is a thing a test reads back rather than a
//! screen somebody has to look at. Whether one particular row is OPEN is the
//! toolkit's own memory, keyed on the row (`super::render`), for the reason
//! bl-83ae's scroll anchor is: a per-row view state that no other fact can be
//! asked for and that no gesture on the wire corresponds to.

/// **How much of a machine's answer stands without being asked for.**
///
/// Two bounds rather than one, because a tool result overruns in two different
/// shapes and either alone lets the other through: a `git log` is six hundred
/// short lines, and a JSON blob is thirty kilobytes on ONE line, which a pane
/// that wraps paints as three hundred rows of glass.
const HEAD_LINES: usize = 6;
/// The other bound, in characters — see [`HEAD_LINES`].
const HEAD_CHARS: usize = 500;

/// **What a folded row shows, and the word on the control that unfolds it.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fold {
    /// The few lines that stand without being asked for.
    pub head: String,
    /// **The word on the control that unfolds it**, which is the act it fires
    /// followed by the two counts a reader is choosing between. One run rather
    /// than a label beside a button: the size of what is hidden is exactly the
    /// thing that makes the control worth pressing, so it belongs on it —
    /// and it is on the glass whenever it matters, which is while the row is
    /// folded. [`UNFOLD`] is what the same control says once it is open.
    pub whole: String,
}

/// **The fold for one machine answer, or `None` where there is nothing to
/// hide.**
///
/// A short result is not folded at all — an expander over four lines is a
/// control that costs a gesture and saves nothing, and the row it would wrap
/// is already the glance it is supposed to be.
pub fn of(content: &str) -> Option<Fold> {
    let head = head(content);
    (head.len() < content.len()).then(|| Fold {
        head,
        whole: whole(content),
    })
}

/// The first [`HEAD_LINES`] lines, and at most [`HEAD_CHARS`] characters of
/// them. Cut on a character boundary, never a byte one.
fn head(content: &str) -> String {
    let lines: String = content.split_inclusive('\n').take(HEAD_LINES).collect();
    lines.chars().take(HEAD_CHARS).collect()
}

/// **What the same control says once the row is open** — the act it fires,
/// which is the other one. Two words rather than one that toggles, on the
/// pin's own rule (`crate::ui::roster::wall`): a control names the act it
/// fires, so an operator never has to work out which way it will go.
pub const UNFOLD: &str = "fold";

/// **The word on the control while the row is folded**: the act, and the two
/// counts a reader is choosing between. The line count is what a scroll costs
/// and the byte count is what the wire carried, and neither derives the other
/// — forty identical paths and one long JSON object are the same kilobytes and
/// nothing like the same read.
fn whole(content: &str) -> String {
    let lines = content.lines().count();
    format!(
        "show all {lines} {}, {} bytes",
        if lines == 1 { "line" } else { "lines" },
        content.len()
    )
}

#[cfg(test)]
mod tests;
