//! **The keyboard walk between frames** (`crate::ui::keys`; DESIGN §4.42 item
//! 7, §4.44 item 2; bl-5b49): whether it owes its selection a place on the
//! glass, the selection it set out from, and the switcher's four acts — open,
//! preview, choose, leave.
//!
//! **The cursor is the selection, and a tab is a choosing.** The arrows select
//! as they walk, so the transcript behind the switcher follows the cursor; but
//! a walk of four rows that opened four tabs was a strip filled by where the
//! operator PASSED. So the walk previews ([`Model::preview`] — the select
//! without the tab) and the strip records only what was chosen: Enter or a
//! click ([`Model::chose`]), which opens the previewed conversation's tab.
//! Esc, or the chord again, leaves without choosing ([`Model::close_switcher`])
//! and puts the preview down — back to the selection the switcher stood over.

use super::Model;

/// **What the walk holds between frames**, per run and never in the place
/// file: both are where the operator's cursor IS, which no read can answer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Walk {
    /// **Whether the list owes its selection a place on the glass**, set by a
    /// keyboard walk and taken by the pane that paints it
    /// ([`Model::revealing`]).
    ///
    /// The keyboard is the only surface that can move a selection out of view:
    /// a click names a row the operator is already looking at, and a scroll IS
    /// the operator deciding what to look at — so a pane that revealed on every
    /// frame would drag the glass back every time they scrolled away.
    pub reveal: bool,
    /// **The selection the switcher stood over** when it opened: what a leave
    /// without a choosing goes back to.
    pub from: Option<String>,
}

impl Model {
    /// **Open the switcher**, or close it: the one toggle Ctrl+X is.
    ///
    /// **Opening it puts down whatever stood** (bl-339d; DESIGN §4.42 item 6,
    /// *one stands at a time*): the chord is the one way to a second overlay
    /// while one is open, and a switcher stacked over a pane handed the
    /// operator back to that pane after they chose a conversation. Each is
    /// put down by its own Esc, so a pane with a step back (the clearing to
    /// its trail, an assignment to its roles) goes all the way down.
    pub fn switch(&mut self) {
        if self.switching {
            self.close_switcher();
        } else {
            self.uncover();
            self.begin_switching();
        }
    }

    /// **Stand the switcher over the selection** — the chord's opening half,
    /// and the whole of what `/conversations` does.
    pub fn begin_switching(&mut self) {
        self.walk.from = self.conversation.clone();
        self.switching = true;
    }

    /// **Stand the cursor on a conversation without opening its tab**: the
    /// walk's act. Everything [`Model::select`] does but the strip — the
    /// transcript and the live tail are the old one's, and go.
    pub fn preview(&mut self, root_id: &str) {
        self.moving_to(Some(root_id));
        self.conversation = Some(root_id.to_owned());
        self.standing = None;
        self.founding = None;
        self.transcript = crate::reply::transcript::Transcript::default();
        self.live = None;
        // **The records pane goes with the conversation it was opened on**
        // (bl-2cf7) — the same rule the tuning pane keeps for its workspace, one
        // noun over (`super::records`).
        self.retire_records();
    }

    /// **A conversation was chosen**, so the switcher has done its work and
    /// the strip records the choice. Called by the click on a conversation's
    /// row, by the `+`, and by Enter on a selection — never by the walk.
    pub fn chose(&mut self) {
        if let Some(root_id) = self.conversation.clone() {
            self.open_tab(&root_id);
        }
        self.walk.from = None;
        self.switching = false;
    }

    /// **Leave without choosing** — Esc, the chord again, or the queue's
    /// door. A walk's preview is put down: the selection goes back to what
    /// the switcher stood over, where that is still a tab of the aimed
    /// workspace, and to nothing where it is not — the walk re-aimed, or the
    /// listing took the tab — because a selection with no tab is a state the
    /// strip cannot paint.
    pub fn close_switcher(&mut self) {
        self.switching = false;
        let from = self.walk.from.take();
        if self.conversation == from {
            return;
        }
        match from.filter(|root_id| self.strip().contains(root_id)) {
            Some(root_id) => self.select(&root_id),
            None => self.deselect(),
        }
    }
}
