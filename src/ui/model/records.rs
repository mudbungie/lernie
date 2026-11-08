//! **The records pane between frames** (bl-2cf7): open or not, and the acts
//! that open and close it.
//!
//! # One state, so it is a word of the context rail's field
//!
//! The tuning pane is a two-state enum because an assignment can be mid-edit;
//! this pane's own state is one draft and no mode — every row on it is the
//! server's answer, filed on [`Model`] beside the roles whether or not the
//! pane is open, and the fork's two words are a bare field (`super::spine`).
//! So the pane is [`Context::Records`], the bare word of the field it shares
//! with the config pane (`super::rail`), and a struct here would be a state
//! machine with one state.
//!
//! # It follows the tuning pane's whole shape, one noun over
//!
//! The subject is the SELECTED CONVERSATION where tuning's is the aimed workspace,
//! and every consequence carries across: the pane opens only where there is a
//! subject; the two reads stand while it is open and only then
//! (`crate::state::Standing::records`), so what it paints is always the
//! server's fact and never this end's prediction; and it closes when the
//! subject moves ([`Model::select`], [`Model::aim_at`]), because a pane about
//! *the selected conversation* left open over a new selection would paint one
//! conversation's records under another's name for a beat.

use super::{Context, Model};

/// **What the records pane has been answered** — its seven reads, held
/// together.
///
/// One struct rather than seven fields on [`Model`], for `super::listing`'s
/// reason one layer over: they are one pane's questions about one subject, and
/// they are retired together the moment that subject moves
/// ([`Model::retire_records`]). Seven options spread across the model would be
/// seven places to remember, and forgetting one paints a conversation's
/// records under another conversation's name.
///
/// Every one is `None` until an answer lands, which is the reading
/// [`Model::roles`]'s option carries: nobody has been answered is not the same
/// claim as a conversation whose loop did nothing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Records {
    /// The steps its loop has taken (§4.18).
    pub steps: Option<crate::reply::steps::Steps>,
    /// What its worktree holds (§4.18).
    pub files: Option<crate::reply::files::Files>,
    /// Its spine — every operable commit, and the cards off them (§4.29).
    pub rail: Option<crate::reply::rail::Rail>,
    /// The config commit governing it (§4.29).
    pub governing: Option<crate::reply::governing::Governing>,
    /// Its own row, whole — the pane's header (§4.32).
    pub agent: Option<crate::reply::agent::Agent>,
    /// Its undelivered mail (§4.32).
    pub mail: Option<Vec<crate::reply::inbox::Row>>,
    /// **One step's records**, and the one of the seven that is POSTED rather
    /// than standing: it is asked about a row, by the control on that row.
    /// Which step it is about is the answer's own `seq`, so nothing here is a
    /// second name for it (`super::deep`).
    pub drilled: Option<crate::reply::step::Step>,
}

impl Model {
    /// **The selected conversation's own row — where the read answered THAT
    /// conversation** (bl-3b46). The read is filed for whatever it answered,
    /// and nothing retires it when the selection is cleared for a start or an
    /// answer lands after a switch; so every reader asks here, and the
    /// question is derived from the selection rather than kept in step with
    /// it. A spend, an offer or a descent read for one conversation is never
    /// painted against another's name, or against none.
    pub fn agent(&self) -> Option<&crate::reply::agent::Agent> {
        self.records
            .agent
            .as_ref()
            .filter(|row| self.conversation.as_deref() == Some(row.agent.as_str()))
    }

    /// **Open the records pane on the selected conversation**, or do nothing
    /// where none is selected. The selection is the gate exactly as the aim is
    /// tuning's: the two reads it stands up are addressed, and a selected
    /// conversation is what an address is.
    pub fn begin_records(&mut self) {
        if self.conversation.is_some() {
            self.context = Some(Context::Records);
        }
    }

    /// **Whether the records pane is standing** — on the context rail or
    /// floating (`super::rail`).
    pub fn recording(&self) -> bool {
        self.context == Some(Context::Records)
    }

    /// **Close it, and only it**: a stale close cannot take down the config
    /// pane that replaced it on the rail. The answers stay, for the same
    /// reason the roles do: the next open on the same conversation is about
    /// the same records, and the standing read replaces them anyway.
    pub fn close_records(&mut self) {
        if self.recording() {
            self.context = None;
        }
    }

    /// **The records go with the conversation they answer** — called by the
    /// two acts that move the subject, so the pane and its rows never outlive
    /// what they are about.
    pub(super) fn retire_records(&mut self) {
        self.close_records();
        // **One assignment, because the answers are one value** ([`Records`]):
        // a pane whose reads are retired field by field is a pane with a read
        // somebody will forget.
        self.records = Records::default();
        // **The draft goes with them** (`super::spine`), which the answers do
        // not need to say: a goal typed for one conversation is a sentence
        // about that one, and a box left standing over a new selection would
        // fire it at whatever is selected next.
        self.forking = crate::ui::Forking::default();
        // **And so does the arming** (`crate::ui::records::cascade`), for a
        // sharper form of the same reason: a name typed to admit ONE
        // conversation's subtree, left standing over the next selection, is an
        // arming for a conversation nobody armed. It cannot fire at one — the
        // control compares the box against the row it is painted beside — but
        // a box that still reads the old name is a control that looks armed.
        self.cascade = String::new();
    }

    /// **Stop the selected conversation and everything under it**, or do
    /// nothing where nothing is aimed at, nothing is selected, or the box does
    /// not hold the name.
    ///
    /// The three gates are one reading rather than three arms of a control, on
    /// `super::spine::Model::post_fork`'s own terms: every state they exclude
    /// is one the pane cannot paint this control in, so they make the states
    /// unreachable rather than merely unlikely.
    ///
    /// **The arming is not spent** (DESIGN §4.20): the refusal is the common
    /// answer here, and clearing the box would charge a retype for the
    /// server's *no*.
    pub fn post_cascade(&mut self) {
        let (Some(aim), Some(agent)) = (self.aim.clone(), self.conversation.clone()) else {
            return;
        };
        let armed = self
            .agent()
            .is_some_and(|row| crate::ui::records::cascade::armed(row, &self.cascade));
        if !armed {
            return;
        }
        self.outbox.push(super::Posted::act(crate::verbs::stop(
            aim.address,
            agent,
            // **The boundary field, at its one site in this window.** litany
            // bl-3114 made every litany stop take its children, so yog retires
            // this field at its next PROTOCOL bump (yog bl-6efc) — and what
            // the bump deletes here is this line.
            true,
        )));
    }
}

#[cfg(test)]
mod tests;
