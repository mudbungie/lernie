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
    /// **How many rows the composer's field stands at**: the operator's own
    /// where they dragged the panel's top edge, else
    /// [`crate::ui::theme::COMPOSER_ROWS`] — which is the whole of what
    /// absence in [`crate::ui::Dragged`] means (DESIGN §4.39).
    ///
    /// It is asked here rather than by the composer because the layout asks it
    /// too: `crate::ui::shell::drag` hands the panel the rows it stood at in
    /// order to measure everything else the panel holds, and two readings of
    /// one number would be two numbers.
    pub fn composer_rows(&self) -> u8 {
        self.dragged.rows.unwrap_or(crate::ui::theme::COMPOSER_ROWS)
    }

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
