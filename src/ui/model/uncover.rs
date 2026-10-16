//! **One overlay stands at a time, read both ways** (DESIGN §4.42 item 7;
//! §4.46 item 3; bl-8c80).
//!
//! Opening the switcher put down whatever stood (bl-339d), and so did
//! opening a dropup (bl-27dd), each with its own loop over [`Model::escape`].
//! Opening a pane by its word did not — the box stands down under a pane, so
//! the word was never typed under one — until the chords reached the words
//! from under anything: Ctrl+, under the switcher opened the settings pane
//! beneath it, and Esc then revealed it. The loop is one function now, and
//! every word fired goes through it ([`crate::ui::slash::fire`]).
//!
//! **The enrollment is the exception it is everywhere**: it covers the
//! window and is only ever left by its own gesture, so nothing is fired
//! under it at all.

use crate::ui::Model;

impl Model {
    /// **Put down whatever stands** — every pane, the switcher, a draft
    /// inside a pane — by the same steps Esc would take, one at a time,
    /// until nothing covers the window. The dropup is not covered and is
    /// left alone; the enrollment is never touched.
    pub fn uncover(&mut self) {
        if self.enroll.is_some() {
            return;
        }
        while self.covered() {
            self.escape();
        }
    }
}
