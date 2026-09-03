//! **The window's own two panes between frames** (bl-40ec; DESIGN §4.21): the
//! engine's verb table, and what a needle found.
//!
//! # Neither has a subject anything on the glass can move
//!
//! `crate::ui::tuning` is about the aimed wall and `crate::ui::records` about
//! the selected conversation, so both are retired when their subject moves.
//! These two are the decision queue's shape (`super::queue`): their ops name
//! no workspace, so the subject is every channel this box holds and no aim or
//! selection can invalidate it. They survive [`super::Model::aim_at`] and
//! [`super::Model::select`] for that reason, which is the same rule those two
//! keep rather than an exception to it.
//!
//! # But the reads are POSTED, not standing, and that is the difference
//!
//! The queue stands on its pane because *what is asking* changes under the
//! operator while they look at it. Neither of these does. A verb table is
//! fixed for the life of an engine build, and a search is an answer to a
//! needle somebody typed — re-asking either on every beat would spend a round
//! trip per channel, forever, on an answer that cannot have changed, and in
//! the search's case would re-scan every store on the box while they are still
//! typing. So each is composed into [`super::Model::outbox`] by the control
//! that asks it, `crate::offframe::poster` fans it over every channel, and the
//! answers replace one channel's section at a time.
//!
//! # The needle is never spent on firing
//!
//! `crate::ui::composer` clears the draft when a deposit is sent, because it
//! was sent. A search is the unmaking's case instead (DESIGN §4.20): refining
//! a needle is the common act, so clearing the box would charge a retype for
//! every second search. What the pane says these rows answer is the engine's
//! own echoed `needle`, not what the box currently holds, so the two cannot
//! disagree while an operator is mid-edit.

use super::Model;
use crate::reply::help::HelpRow;
use crate::reply::search::Found;
use crate::ui::Channel;

/// **Which of the window's own two panes is standing**, if either.
///
/// One field and not two flags. The two are mutually exclusive on the glass —
/// both cover the conversation, and the shell paints one — so a pair of bools
/// makes *both open* a representable state that only the paint order resolves,
/// which is two representations of one fact. It is also the reframe clippy's
/// `struct_excessive_bools` asks for by name, taken rather than suppressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lookup {
    /// The engines' own verb tables (`crate::ui::commands`).
    Commands,
    /// Text found across everything they can see (`crate::ui::find`).
    Finding,
}

/// **One channel's answer to *what do you answer to***, and the channel it
/// came down as the client's own stamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pages {
    pub channel: Channel,
    pub rows: Vec<HelpRow>,
}

/// **One channel's answer to a needle**, stamped the same way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hits {
    pub channel: Channel,
    pub found: Found,
}

impl Model {
    /// **Open the commands pane and ask for the table.** It takes no subject,
    /// so nothing gates it: *what can I ask for* is answerable from an
    /// unaimed, unselected seat, and that is the seat most likely to be asking.
    pub fn begin_commands(&mut self) {
        self.lookup = Some(Lookup::Commands);
        self.outbox
            .push(super::Posted::read(crate::verbs::window::help()));
    }

    /// **Whether that pane is the one standing.**
    pub fn commanding(&self) -> bool {
        self.lookup == Some(Lookup::Commands)
    }

    /// File one channel's verb table, replacing what that channel last said
    /// and leaving every other channel standing — REMOTE §8.2's *"a refusal is
    /// one entry's, never the set's"*, the shape the roster already keeps.
    pub(super) fn paged(&mut self, channel: &Channel, rows: Vec<HelpRow>) {
        let answered = Pages {
            channel: channel.clone(),
            rows,
        };
        match self
            .pages
            .iter_mut()
            .find(|held| held.channel.name == channel.name)
        {
            Some(held) => *held = answered,
            None => self.pages.push(answered),
        }
    }

    /// **Open the find pane.** It asks nothing: there is no needle yet, and a
    /// search with an empty one is a scan of every store on the box for
    /// nothing.
    pub fn begin_finding(&mut self) {
        self.lookup = Some(Lookup::Finding);
    }

    /// **Whether that pane is the one standing.**
    pub fn finding(&self) -> bool {
        self.lookup == Some(Lookup::Finding)
    }

    /// **Put whichever of the two is standing down.** One door for both, and
    /// for Escape as well: closing either means the same thing, and two
    /// spellings of it would be two places a later pane has to be added to.
    ///
    /// The rows and the needle stay, for the reason the roles and the records
    /// do: the next open is about the same table, and an answer replaces them
    /// when the next ask lands.
    pub fn close_lookup(&mut self) {
        self.lookup = None;
    }

    /// **Whether there is a needle to search for**, which is what enables the
    /// control rather than what hides it (DESIGN §4.20's enablement rule): the
    /// parameter is missing, not the subject.
    ///
    /// Trimmed, because whitespace is not a needle and a search for it would
    /// match every field on the box.
    pub fn needled(&self) -> bool {
        !self.needle.trim().is_empty()
    }

    /// **Spend the needle**, or do nothing where there is none. The box is not
    /// cleared — see the module doc.
    pub fn post_search(&mut self) {
        if self.needled() {
            self.outbox.push(super::Posted::read(crate::verbs::search(
                self.needle.trim().to_owned(),
            )));
        }
    }

    /// File one channel's hits, on [`Model::paged`]'s own terms.
    pub(super) fn hit(&mut self, channel: &Channel, found: Found) {
        let answered = Hits {
            channel: channel.clone(),
            found,
        };
        match self
            .found
            .iter_mut()
            .find(|held| held.channel.name == channel.name)
        {
            Some(held) => *held = answered,
            None => self.found.push(answered),
        }
    }

    /// **Ask every channel for its roster again** — the one gesture that
    /// reaches the view the standing `workspaces` read populates (yog's
    /// `docs/PARITY.md` §2).
    ///
    /// It clears nothing. Each answer replaces its own channel's chunk when it
    /// lands (`super::absorb`), and a channel that cannot be reached says so
    /// on its own section — which is the other half of what this control is
    /// for: until it existed that sentence appeared only on a beat nobody
    /// could ask for.
    pub fn refresh_roster(&mut self) {
        self.outbox
            .push(super::Posted::read(crate::verbs::workspaces()));
    }
}

#[cfg(test)]
mod tests;
