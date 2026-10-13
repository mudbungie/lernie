//! **The two questions asked of the model that no pane owns** — the composer's
//! row count, and whether a keyboard walk owes its selection a place on the
//! glass.
//!
//! Split from [`super`] at the 300-line cap on the seam that file's own doc
//! draws: there is the snapshot a frame reads, and here is what is asked of
//! it. The struct grows when a pane learns to hold something; this grows when
//! a question turns out to belong to the window rather than to one pane —
//! which has happened twice, and both times because two panes needed the same
//! answer.

use super::Model;

impl Model {
    /// **Whether the list must bring its selection onto the glass this
    /// frame**, answered once: the flag is taken, so a stale request cannot
    /// fight the next frame's scroll.
    ///
    /// It used to ask WHICH pane, because two of them held a list and only the
    /// focused one owed its selection a place. There is one list since DESIGN
    /// §4.39's fold (bl-b9a3), so the question is the flag.
    pub fn revealing(&mut self) -> bool {
        std::mem::take(&mut self.reveal)
    }
}
