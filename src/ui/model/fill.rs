//! **The box a menu item sends the operator to** (bl-dbc9): which of the
//! composer's two parameter boxes wants filling, and the one door that names a
//! conversation and asks for it.
//!
//! # Why a menu item ever needs this
//!
//! `crate::ui::convs::menu` fires the acts on a conversation row that take the
//! wall and the conversation and nothing else. Two of the acts on that row take
//! a **third** parameter — `flag`'s reason and `delete-agent`'s arming — and
//! each has a box of its own on the composer's second row
//! (`crate::ui::composer::acts`). A menu cannot hold that box: the composer is
//! a bottom panel that stands down off the conversation's own column in the
//! narrow shape (`crate::ui::shell`), so a box in a list row's menu would be a
//! parameter reachable at one width and not the other, and firing without one
//! would be an act composed from a control that never asked for it.
//!
//! So the menu item is a **navigation**: it names the conversation, goes to
//! where the box is, and puts the cursor in it. It crosses no wire and carries
//! no `act:` token, which is `Model::go_to`'s division exactly (§4.16: an aim,
//! a selection and a focus are views, and views are out of the parity
//! contract).
//!
//! # It is taken once, by the frame that paints the box
//!
//! [`Model::filling`] is `Model::revealing`'s shape one noun over: the field is
//! set by the surface that asked and **taken** by the surface that answers, so
//! a request cannot fight the next frame's focus. Taking it in one read rather
//! than one per box is what keeps a frame that paints the row from leaving half
//! a request standing.
//!
//! In the broad shape the composer paints later in the same frame as the list,
//! so the cursor lands on the keypress. In the narrow shape the list is the
//! central panel and the composer is already behind it, so it lands on the next
//! frame — which is the reason this is a field at all rather than a call.

use super::Model;

/// **Which of the composer's two parameter boxes wants the cursor.**
///
/// Two variants and no third: these are exactly the acts on a conversation
/// whose extra parameter has a box of its OWN. `message` and `interrupt` share
/// the composer's one draft, so pointing at it would name neither — which is
/// why neither is on the row's menu (`crate::ui::convs::menu`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fill {
    /// The words a flag is raised with (`crate::ui::composer::acts::WHY`).
    Reason,
    /// The name that admits a conversation's descendants into its deletion
    /// (`crate::ui::composer::acts::ARM`).
    Arming,
}

impl Model {
    /// **Go to the box that fills an act on `root_id`.**
    ///
    /// Three doors and no fourth: the selection, because the composer acts on
    /// the selected conversation and a box filled against another row would
    /// compose a gesture about something else; the column, because in the
    /// narrow shape the composer is on the conversation's own column and
    /// nowhere else; and the request itself.
    ///
    /// It unmakes nothing and asks for nothing. That is the whole of the row
    /// menu's answer to DESIGN §4.20 — *a routine surface is one an operator
    /// moves through quickly, and a mis-aimed click there must not be able to
    /// land on a destruction* — read on a control rather than on a pane.
    pub fn fill_in(&mut self, root_id: &str, fill: Fill) {
        self.select(root_id);
        self.column = crate::ui::Column::Conversation;
        self.fill = Some(fill);
    }

    /// **Which box this frame owes the cursor**, answered once.
    ///
    /// Taken rather than read, for [`Model::revealing`]'s reason: a request
    /// that survived the frame that answered it would drag the focus back every
    /// time the operator moved it.
    pub fn filling(&mut self) -> Option<Fill> {
        self.fill.take()
    }
}

#[cfg(test)]
mod tests;
