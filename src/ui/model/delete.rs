//! **A conversation's deletion** (DESIGN §4.44 item 1; §4.42 item 5;
//! bl-e837): the one door the bar's trashcan and `/delete` share, and the
//! confirmation that stands between a click and the act.
//!
//! **The act is the wire's `delete-agent`, armed by the conversation's own
//! name** — §4.42 item 5's rule, which `/delete` keeps by refusing any other
//! word (`crate::ui::slash::fire::wanting`). The trashcan was clicked on the
//! row that bears the name, so there is no word to compare and the seat
//! types it itself; one function composes the envelope for both doors, so
//! what `/delete <name>` sends and what the trashcan sends cannot drift.
//!
//! **The confirmation holds the root id and not the name.** A name is what
//! the listing says this frame; the id is what the engine deletes, and a
//! listing that re-answers while the question stands renames nothing
//! underneath it.

use super::Model;

impl Model {
    /// **The name the engine compares** for `root_id`: the listing's display
    /// for it, or the id itself where the listing no longer has the row.
    pub fn name_of(&self, root_id: &str) -> String {
        self.rows()
            .into_iter()
            .find(|row| row.root_id == root_id)
            .map_or_else(|| root_id.to_owned(), |row| row.display)
    }

    /// **Ask before deleting `root_id`**: the confirmation stands until it
    /// is answered (`crate::ui::bar::confirm`) — unless the `confirm` knob
    /// is off (DESIGN §4.46 item 4), when the door deletes at once as a
    /// Ctrl-click does.
    pub fn begin_deleting(&mut self, root_id: &str) {
        if self.settings.on(crate::settings::Toggle::Confirm) {
            self.deleting = Some(root_id.to_owned());
        } else {
            self.post_delete(root_id);
        }
    }

    /// **Spare it**: the confirmation goes and nothing is sent.
    pub fn spare(&mut self) {
        self.deleting = None;
    }

    /// **Delete `root_id`**: compose `delete-agent` on the aimed wall, armed
    /// by the name, and put the confirmation down whichever door fired it.
    /// Nothing is composed with no wall aimed — a row is a wall's, so with
    /// none aimed there is none to delete.
    pub fn post_delete(&mut self, root_id: &str) {
        self.deleting = None;
        let Some(aim) = self.aim.clone() else {
            return;
        };
        let name = self.name_of(root_id);
        let gesture = crate::verbs::delete_agent(aim.address, root_id.to_owned(), name);
        self.outbox.push(super::Posted::act(gesture));
    }
}

#[cfg(test)]
mod tests;
