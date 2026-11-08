//! **The conversations kept at the bar's top** (DESIGN §4.48 item 5): the
//! baseline's *Pinned*, held per seat in the place file (`crate::place`) as
//! the root ids under each workspace (`crate::ui::bar::Bar::kept`).
//!
//! **It is not §4.25's pin**, which is the wire's `/pin` by workspace rank —
//! an assertion about the world. This is an arrangement of one listing on
//! one seat, §4.39's class of fact, and the words keep them apart: the field
//! is `kept`, the help says *keeps this conversation at the top of this bar*.
//! **An id the listing no longer knows leaves with it** (bl-6dfd's rule, the
//! tabs' own, read one field over).

use super::{Model, Tab};

impl Model {
    /// **The aimed workspace's kept conversations**, by root id, in the order
    /// they were kept.
    pub fn kept(&self) -> Vec<String> {
        self.bar
            .kept
            .iter()
            .filter(|tab| Some(&tab.aim) == self.aim.as_ref())
            .map(|tab| tab.root_id.clone())
            .collect()
    }

    /// **Keep `root_id` at the bar's top, or stop keeping it** — under the
    /// aim, and nothing with none.
    pub fn toggle_kept(&mut self, root_id: &str) {
        let Some(aim) = self.aim.clone() else {
            return;
        };
        let tab = Tab {
            aim,
            root_id: root_id.to_owned(),
        };
        if self.bar.kept.contains(&tab) {
            self.bar.kept.retain(|kept| *kept != tab);
        } else {
            self.bar.kept.push(tab);
        }
    }

    /// **The listing answered: the kept ids it no longer knows leave with it**,
    /// under the workspace it answered and no other.
    pub(super) fn prune_kept(&mut self) {
        let Some(answered) = self.answered.clone() else {
            return;
        };
        let convs = &self.convs;
        self.bar.kept.retain(|tab| {
            tab.aim != answered || convs.iter().any(|row| row.root_id == tab.root_id)
        });
    }
}

#[cfg(test)]
mod tests;
