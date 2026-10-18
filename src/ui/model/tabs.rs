//! **The tabs between frames** (DESIGN §4.44 item 2; bl-03c1): which
//! conversations the operator has open, and the four things that change it.
//!
//! **Open is a navigation the operator performed** — [`Model::opened`]'s
//! class of fact, which no read can answer — so it is held, per run and never
//! in the place file. It is filled by [`Model::select`] and by nothing else:
//! every door that selects opens a tab by selecting, so there is no second act
//! that opens one. It is emptied by the close, and by the server: a tab whose
//! conversation the aimed workspace's listing no longer knows leaves with it, and a
//! selection the server put down (bl-6dfd) is a tab put down.

use super::start::Phase;
use super::{Aim, Model};

/// **One open conversation**, under the workspace it was selected on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tab {
    /// The workspace it was opened under: the strip paints the aimed workspace's.
    pub aim: Aim,
    /// The id every gesture addresses it with.
    pub root_id: String,
}

impl Model {
    /// **The aimed workspace's open conversations**, by root id, in the order they
    /// were first selected — what the strip paints.
    pub fn strip(&self) -> Vec<String> {
        self.tabs
            .iter()
            .filter(|tab| Some(&tab.aim) == self.aim.as_ref())
            .map(|tab| tab.root_id.clone())
            .collect()
    }

    /// **Open `root_id`'s tab** under the aim, unless it is open already.
    pub(super) fn open_tab(&mut self, root_id: &str) {
        let Some(aim) = self.aim.clone() else {
            return;
        };
        let tab = Tab {
            aim,
            root_id: root_id.to_owned(),
        };
        if !self.tabs.contains(&tab) {
            self.tabs.push(tab);
        }
    }

    /// **Close one of the aimed workspace's tabs.** Where it was the selected one,
    /// the neighbour to its left is selected — to its right at the left end —
    /// and where it was the last, the selection is put down to the workspace.
    pub fn close_tab(&mut self, root_id: &str) {
        let strip = self.strip();
        let Some(at) = strip.iter().position(|id| id == root_id) else {
            return;
        };
        self.shut(root_id);
        if self.conversation.as_deref() != Some(root_id) {
            return;
        }
        let left = at.checked_sub(1).and_then(|left| strip.get(left));
        if let Some(next) = left.or_else(|| strip.get(at + 1)) {
            self.select(next);
        } else {
            self.moving_to(None);
            self.deselect();
        }
    }

    /// **Drop the aimed workspace's tab on `root_id`**, whatever is selected.
    pub(super) fn shut(&mut self, root_id: &str) {
        let aim = self.aim.clone();
        self.tabs
            .retain(|tab| !(Some(&tab.aim) == aim.as_ref() && tab.root_id == root_id));
    }

    /// **A start's tab takes the root the server minted** ([`Model::resolve`]):
    /// it was opened under the claimed name, and it is the same conversation.
    pub(super) fn rename_tab(&mut self, name: &str, root_id: &str) {
        let renamed: Vec<Tab> = self
            .tabs
            .drain(..)
            .map(|tab| Tab {
                root_id: if tab.root_id == name {
                    root_id.to_owned()
                } else {
                    tab.root_id
                },
                aim: tab.aim,
            })
            .collect();
        for tab in renamed {
            if !self.tabs.contains(&tab) {
                self.tabs.push(tab);
            }
        }
    }

    /// **The listing answered: the tabs it no longer knows leave with it** —
    /// only under the workspace it answered ([`Model::answered`]), and never a
    /// start's claimed name, which no listing knows until it resolves.
    pub(super) fn prune_tabs(&mut self) {
        let Some(answered) = self.answered.clone() else {
            return;
        };
        let claimed = match self.start.as_ref().map(|start| &start.phase) {
            Some(Phase::Started(name)) => Some(name.clone()),
            _ => None,
        };
        let convs = &self.convs;
        self.tabs.retain(|tab| {
            tab.aim != answered
                || claimed.as_deref() == Some(tab.root_id.as_str())
                || convs.iter().any(|row| row.root_id == tab.root_id)
        });
    }
}

#[cfg(test)]
mod tests;
