//! **A dropup between frames** (DESIGN §4.42, item 11): which selector on
//! the rule is open, where its cursor stands, and whether the read it posted
//! has been answered.
//!
//! A selector opens on what this seat already holds — the roster's walls,
//! the providers read, one provider's models, the provider row's levels —
//! and posts the read that refreshes that list in the same gesture. The list
//! is the engine's answer and lives where every answer lives; what is held
//! here is only the question of whether the refresh has come back, because
//! that is what the mark on the segment says.
//!
//! **A pick is the door the role tuning fires for the worker role**, or the
//! switcher's own aim for a wall — never a second spelling of either.

use super::{Aim, Model, Posted};
use crate::reply::Reply;
use crate::reply::roles::RoleRow;

/// The role a start runs on, and the one the rule's selectors set.
pub const WORKER: &str = "worker";

/// **One selector on the rule**, left to right.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selector {
    Workspace,
    Provider,
    Model,
    Effort,
    Priority,
}

/// **Whether the refresh a dropup posted has been answered.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reload {
    /// Posted, and nothing has come back.
    Asked,
    /// The answer landed; the list is the engine's latest.
    Landed,
}

/// **An open dropup.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dropup {
    pub selector: Selector,
    /// The provider row a model list is for — the worker's, or the one just
    /// picked on the provider list, because a provider alone is half an
    /// assignment and the write takes the pair.
    pub provider: Option<String>,
    pub cursor: usize,
    /// `None` where nothing could be asked (a model list with no provider).
    pub reload: Option<Reload>,
}

/// **What picking a row does.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pick {
    Wall(Aim),
    /// `+ new workspace` on the named channel (bl-673e; DESIGN §4.44).
    Found(String),
    Provider(String),
    Model(String),
    Effort(Option<String>),
    Priority(bool),
}

/// **One row of a dropup**: its words, whether it is what is in force, and
/// what picking it does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    pub text: String,
    pub current: bool,
    pub pick: Pick,
}

impl Selector {
    /// **The op of the read this selector posts.**
    fn op(self) -> &'static str {
        match self {
            Self::Workspace => crate::verbs::WORKSPACES.word,
            Self::Model => crate::verbs::MODELS.word,
            Self::Provider | Self::Effort | Self::Priority => crate::verbs::PROVIDERS.word,
        }
    }

    /// Whether `reply` is the answer to the read this selector posts.
    fn answered_by(self, reply: &Reply) -> bool {
        match self {
            Self::Workspace => matches!(reply, Reply::Workspaces(_)),
            Self::Model => matches!(reply, Reply::Models(_)),
            Self::Provider | Self::Effort | Self::Priority => {
                matches!(reply, Reply::Providers(_))
            }
        }
    }
}

impl Model {
    /// **The worker role's row**, where the roles read has answered one.
    pub fn worker(&self) -> Option<&RoleRow> {
        self.roles.as_ref()?.iter().find(|row| row.role == WORKER)
    }

    /// **Open `selector`'s dropup**, or close it where it is the one open —
    /// the toggle a click on its segment is.
    ///
    /// **Opening one puts down whatever stands** (bl-27dd), as the switcher's
    /// chord does (`Model::switch`, bl-339d): `/model` and Ctrl+O reach it
    /// from under a pane, and a dropup opened under a cover was closed by the
    /// cover the same frame — the chord did nothing. The enrollment is the
    /// exception it is everywhere.
    pub fn drop_up(&mut self, selector: Selector) {
        if self.dropup.as_ref().is_some_and(|d| d.selector == selector) {
            self.dropup = None;
            return;
        }
        if self.enroll.is_some() {
            return;
        }
        self.uncover();
        let provider = self.worker().map(|row| row.provider.clone());
        self.open_on(selector, provider);
    }

    /// Open a dropup on `provider`'s rows, its cursor on what is in force,
    /// and post the read that refreshes it.
    fn open_on(&mut self, selector: Selector, provider: Option<String>) {
        let mut open = Dropup {
            selector,
            provider,
            cursor: 0,
            reload: None,
        };
        let asked = self.ask_for(&open);
        open.reload = asked.then_some(Reload::Asked);
        self.dropup = Some(open);
        let current = self.choices().iter().position(|choice| choice.current);
        if let Some(open) = &mut self.dropup {
            open.cursor = current.unwrap_or(0);
        }
    }

    /// **Post the read that refreshes `open`'s list**, answering whether one
    /// could be asked.
    fn ask_for(&mut self, open: &Dropup) -> bool {
        let address = self.aim.as_ref().map(|aim| aim.address.clone());
        let read = match (open.selector, address) {
            (Selector::Workspace, _) => crate::verbs::workspaces(),
            (Selector::Model, Some(address)) => match &open.provider {
                Some(provider) => crate::verbs::models(address, provider.clone()),
                None => return false,
            },
            (_, Some(address)) => crate::verbs::providers(address),
            (_, None) => return false,
        };
        self.outbox.push(Posted::read(read));
        true
    }

    /// **Close the dropup**, picking nothing.
    pub fn close_dropup(&mut self) {
        self.dropup = None;
    }

    /// **Move the open dropup's cursor** by `step`, held to its rows.
    pub fn walk_dropup(&mut self, step: isize) {
        let last = self.choices().len().saturating_sub(1);
        if let Some(open) = &mut self.dropup {
            open.cursor = open.cursor.saturating_add_signed(step).min(last);
        }
    }

    /// **Pick the row at `at`**: fire its door and close the dropup — or, for
    /// a provider, open the model list on it, since the write takes the pair.
    pub fn pick(&mut self, at: usize) {
        let Some(choice) = self.choices().into_iter().nth(at) else {
            return;
        };
        let asked = self.dropup.take().and_then(|open| open.provider);
        match choice.pick {
            Pick::Wall(aim) => {
                self.engines.opening(&aim.channel);
                self.aim_at(&aim.channel, &aim.address);
            }
            Pick::Found(channel) => self.begin_founding(&channel),
            Pick::Provider(name) => self.open_on(Selector::Model, Some(name)),
            Pick::Model(id) => {
                if let Some(provider) = asked {
                    self.post_model(WORKER, provider, id);
                }
            }
            Pick::Effort(level) => self.post_effort(WORKER, level),
            Pick::Priority(on) => self.post_priority(WORKER, on),
        }
    }

    /// **A refusal of the read the open dropup posted puts its reload
    /// down**: nothing is coming, so the mark goes back to rest and the
    /// notice says why — a reload that never ends would say the opposite.
    pub(super) fn refused(&mut self, op: &str, read: &crate::reply::Read) {
        use crate::reply::Read;
        if let Some(open) = &mut self.dropup
            && open.selector.op() == op
            && matches!(read, Read::Refusal(_) | Read::Unreadable(_))
        {
            open.reload = None;
        }
    }

    /// **Take one answer's news for the open dropup**: the read it posted
    /// has come back, and a model listing is filed under the row it asked.
    pub(super) fn landing(&mut self, reply: &Reply) {
        let Some(open) = &mut self.dropup else {
            return;
        };
        if !open.selector.answered_by(reply) {
            return;
        }
        open.reload = Some(Reload::Landed);
        if let (Reply::Models(_), Some(provider)) = (reply, open.provider.clone()) {
            self.offering(&provider, crate::reply::Read::Answer(reply.clone()));
        }
    }
}

pub mod choices;

#[cfg(test)]
mod tests;
