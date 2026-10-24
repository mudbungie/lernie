//! **One row of the chat pane** — the shape [`super`] projects every entry
//! into, split out of it at the design-time budget (bl-1d79). The projection
//! stays in [`super`]; this is only what a row is made of and the few ways
//! one is born.

use super::{FAILED, Kind, RETURNED, pair, role};
use crate::ui::chat::Fold;
use crate::ui::theme::{Bubble, Speaker};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// **The transcript entry this row was projected from**, by its name —
    /// the key a notch's seat names (`reply/rail` `row`), so a procedure
    /// run's marker (§4.47) finds the row it stands above. Empty on a row no
    /// entry made: the operator's own just-sent words.
    pub entry: String,
    pub who: String,
    pub said: String,
    /// **Whose voice this is, told by weight and never by hue** (STYLE §5,
    /// `theme::speaker`): the ink of the rule beside the block and of the
    /// header over it. It is decided HERE, where the entry is read, because
    /// the paint knows a row and only this knows what kind of entry made one.
    pub weight: Speaker,
    /// **Whether this row is a failure.** The one thing a transcript says that
    /// is a STATE rather than a speaker — so it is the one row whose rule is
    /// an accent (`State::Error`) instead of a weight, and colour still means
    /// state.
    pub failed: bool,
    /// What this row hides while it is folded, or `None` where it hides
    /// nothing — see [`fold`].
    pub fold: Option<Fold>,
    /// Said, or a step of the trail — see [`Kind`].
    pub kind: Kind,
    /// **The tool-use id that meets a call with its result** ([`pair`]) —
    /// carried for the pairing and never painted: an id is the wire's
    /// bookkeeping, not anything the operator reads.
    pub call: Option<String>,
    /// **Whether the server has yet to acknowledge it** ([`sent`]): the one
    /// row whose words wear the pending ink rather than the body's.
    pub pending: bool,
    /// **The fill this row stands on** (§4.45 item 7), decided here and not
    /// by the painter — see [`role`].
    pub bubble: Bubble,
    /// **Whether this row belongs to the turn that has not settled** — what a
    /// trail's fold reads for its default (§4.45 item 5).
    pub live: bool,
}

impl Row {
    /// **A row that hides nothing and did not fail** — everything a person, a
    /// model or this seat itself put on the glass. One constructor rather than
    /// a `fold: None` at eight sites, because a row that hides something is
    /// the exception and naming the exception is what keeps it visible.
    pub(in crate::ui::chat) fn plain(who: String, said: String, weight: Speaker) -> Self {
        Self {
            entry: String::new(),
            who,
            said,
            weight,
            failed: false,
            fold: None,
            kind: Kind::Said,
            call: None,
            pending: false,
            bubble: role::born(weight, Kind::Said),
            live: false,
        }
    }

    /// The same row as a step of the trail.
    /// Stamp the entry the row came from.
    pub(in crate::ui::chat) fn from(mut self, entry: &str) -> Self {
        entry.clone_into(&mut self.entry);
        self
    }

    pub(in crate::ui::chat) fn as_(mut self, kind: Kind) -> Self {
        self.kind = kind;
        self.bubble = role::born(self.weight, kind);
        self
    }

    /// **A call, as the trail names it**: `name(<head of its input>)`, the
    /// id kept for the pairing and off the glass.
    pub(in crate::ui::chat) fn call(name: &str, input: &str, id: &str) -> Self {
        Self {
            call: Some(id.to_owned()),
            ..Self::plain(pair::head(name, input), String::new(), Speaker::Model).as_(Kind::Call)
        }
    }

    /// **A result**, by the one word for how it ended.
    pub(in crate::ui::chat) fn result(
        id: &str,
        said: String,
        failed: bool,
        fold: Option<Fold>,
    ) -> Self {
        Self {
            entry: String::new(),
            who: if failed { FAILED } else { RETURNED }.to_owned(),
            said,
            // A tool answers for the model, not as it: a peer's weight.
            weight: Speaker::Peer,
            failed,
            fold,
            kind: Kind::Result,
            call: Some(id.to_owned()),
            pending: false,
            bubble: Bubble::Trail,
            live: false,
        }
    }
}
