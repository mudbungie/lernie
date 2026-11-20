//! **The machine a provider's row is on** (DESIGN §4.44 item 3; yog's
//! `docs/REMOTE.md` §5.7) — read off the row's own fields, and the two lists
//! that reading parts: the machines, and one machine's providers.
//!
//! A role's provider is a bare `<row>` — the server's own brazen — or
//! `<row>@<client>`: that row on a machine registered in the workspace. The
//! providers read lists both kinds in one table, so the table is every
//! machine's rows end to end. The rule parts it: the **machine** selector
//! lists the server and each client a row is on, and the provider selector
//! lists one machine's rows by the name that machine gives them.
//!
//! # The server says which machine, and this seat never reads a name
//!
//! Which client a name selects is the server's reading, made against the
//! workspace's registered clients, and neither half of a name is barred from
//! carrying the joiner — so no rule a seat could hold reads it right. The
//! server states its reading instead (REMOTE §5.7, edition 30): every row
//! that names a provider carries the client it is on and its own spelling
//! there ([`Place`]), and a row carrying neither is the server's own. That
//! is all this module reads. Against a server older than the fields every
//! row is therefore the server's, under its whole name — nothing hidden and
//! nothing filed under a machine on a guess.
//!
//! # The machine is derived, and held nowhere
//!
//! The name in force is the role's (`roles`), the names on offer are the
//! table's (`providers`), and each says its own machine. The one fact held is
//! which machine an open provider list is browsing
//! (`super::Dropup::machine`), which dies with the list. The write stays the
//! whole name: a pick carries the table's own spelling and this module only
//! decides what is painted.

use super::{Choice, Model, Pick};
use crate::reply::providers::Place;

/// **What the server itself is called among the machines** — the rows it
/// answers from its own brazen, which no client is stated for.
pub(crate) const SERVER: &str = "server";

/// A machine's word on the glass: the client's name, or [`SERVER`].
pub(crate) fn word(machine: Option<&str>) -> &str {
    machine.unwrap_or(SERVER)
}

impl Model {
    /// Every provider this seat holds, by name and where it is: the table's,
    /// then the worker's own where the table does not name it (bl-67aa — a
    /// worker bound to a preset nobody named, or to a row on a machine that
    /// is asleep, is on no table row, and the lists still say where it
    /// stands).
    fn held(&self) -> Vec<(String, Option<Place>)> {
        let mut held: Vec<(String, Option<Place>)> = self
            .providers
            .iter()
            .flatten()
            .map(|row| (row.name.clone(), row.on.clone()))
            .collect();
        if let Some(worker) = self.worker()
            && !held.iter().any(|(name, _)| *name == worker.provider)
        {
            held.insert(0, (worker.provider.clone(), worker.on.clone()));
        }
        held
    }

    /// The client the held provider `name` is on, or `None` for the server's
    /// own — and for a name this seat holds no row of.
    pub(super) fn machine_of(&self, name: &str) -> Option<String> {
        let on = self.held().into_iter().find(|(held, _)| held == name)?.1;
        on.map(|at| at.client)
    }

    /// **The machines**: the server, always and first, then each client a
    /// held row is on, in the table's order.
    pub(super) fn machine_rows(&self) -> Vec<Choice> {
        let at = self.worker().map(|w| Place::machine(w.on.as_ref()));
        let mut machines: Vec<Option<String>> = vec![None];
        for (_, on) in self.held() {
            let machine = Place::machine(on.as_ref());
            if !machines.contains(&machine) {
                machines.push(machine);
            }
        }
        machines
            .into_iter()
            .map(|machine| Choice {
                text: word(machine.as_deref()).to_owned(),
                current: at.as_ref() == Some(&machine),
                pick: Pick::Machine(machine),
            })
            .collect()
    }

    /// **One machine's providers**, each said as that machine spells it and
    /// picked by the table's own whole name.
    pub(super) fn provider_rows(&self, machine: Option<&str>) -> Vec<Choice> {
        let worker = self.worker().map(|w| w.provider.as_str());
        self.held()
            .into_iter()
            .filter(|(_, on)| Place::machine(on.as_ref()).as_deref() == machine)
            .map(|(name, on)| Choice {
                current: worker == Some(name.as_str()),
                text: Place::shown(on.as_ref(), &name),
                pick: Pick::Provider(name),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests;
