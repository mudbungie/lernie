//! **Removing an entry, between frames** (DESIGN §4.52): which one is armed,
//! which were confirmed, and what a removal's return does to the roster.
//!
//! The act writes this box's disk, so it is never the frame's — the register
//! page's shape (`super::register`), run the other way. The first click arms
//! one entry; the second moves its name to [`Forgetting::asked`]; the next
//! settle hands it to the poster (`crate::state::Link::settle`), and the
//! outcome comes back the same way.
//!
//! **Only an entry is ever removed.** This box's own server is not one — it
//! is the flat root, held without a name — and a channel is an entry exactly
//! where it carries the name its workspace bears on its host.

use super::Model;

/// Where a removal stands.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Forgetting {
    /// The entry the first click named, awaiting the second.
    pub armed: Option<String>,
    /// Entries confirmed and not yet taken by the poster.
    pub asked: Vec<String>,
    /// Why the last removal did not happen.
    pub refused: Option<String>,
}

impl Model {
    /// **The first click**: arm `name`'s removal, disarming any other.
    pub fn arm_forget(&mut self, name: &str) {
        self.servers.forgetting.armed = Some(name.to_owned());
    }

    /// The way out: nothing is armed.
    pub fn disarm_forget(&mut self) {
        self.servers.forgetting.armed = None;
    }

    /// **The second click**: ask for the armed entry's removal. Answers
    /// whether anything was armed.
    pub fn forget(&mut self) -> bool {
        let armed = self.servers.forgetting.armed.take();
        self.servers.forgetting.asked.extend(armed.clone());
        armed.is_some()
    }

    /// **The removal came back.** An entry that is gone from the disk is gone
    /// from the roster on the same frame, and nothing stays aimed down a
    /// channel this box no longer holds; a refusal leaves every row standing
    /// and is said on the page.
    pub fn forgotten(&mut self, name: &str, outcome: Result<(), String>) {
        self.servers.forgetting.refused = outcome.err();
        if self.servers.forgetting.refused.is_some() {
            return;
        }
        self.roster
            .retain(|chunk| chunk.channel.name != name || chunk.channel.named_there.is_none());
        if self.aim.as_ref().is_some_and(|aim| aim.channel == name) {
            self.aim = None;
            self.deselect();
        }
    }
}

#[cfg(test)]
mod tests;
