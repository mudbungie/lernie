//! **The machine a provider's row is on** (DESIGN §4.44 item 3; yog's
//! `docs/REMOTE.md` §5.7) — read off the row's own name, and the two lists
//! that reading parts: the machines, and one machine's providers.
//!
//! A role's provider is a bare `<row>` — the server's own brazen — or
//! `<row>@<client>`: that row on a machine registered in the workspace. The
//! providers read lists both kinds in one table, so the table is every
//! machine's rows end to end. The rule parts it: the **machine** selector
//! lists the server and each client a row names, and the provider selector
//! lists one machine's rows by the name that machine gives them.
//!
//! # The machine is derived, and held nowhere
//!
//! The name in force is the role's (`roles`), the names on offer are the
//! table's (`providers`), and the machine is [`of`] either. The one fact held
//! is which machine an open provider list is browsing
//! (`super::Dropup::machine`), which dies with the list. The write stays the
//! qualified name: a pick carries the table's own spelling and this module
//! only decides what is painted.
//!
//! # The reading is the LAST joiner, and that is not the server's rule
//!
//! The server reads the client off a name by matching the workspace's
//! *registered* clients, the longest that fits (REMOTE §5.7), because neither
//! half is barred from carrying the joiner. This seat cannot do that from the
//! listing: a `providers` row states its name and no client, and the
//! registered set is another read (`clients`) that stands only while its page
//! is open. So [`split`] cuts at the last joiner, which agrees with the server
//! wherever no client's name carries one and none of the server's own rows
//! does. Where one does, the cut is wrong **on the glass only** — a row filed
//! under the wrong machine, its words cut in the wrong place — and never on
//! the wire, since nothing here composes a name. The remedy is the server's:
//! a `client` field on the row would make this module a field read.

use super::{Choice, Model, Pick};

/// What joins a row to the machine it is on.
pub(crate) const JOINER: char = '@';
/// **What the server itself is called among the machines** — the rows it
/// answers from its own brazen, which no name qualifies.
pub(crate) const SERVER: &str = "server";

/// **A provider's name parted**: the row as its machine spells it, and the
/// client it is on — `None` for the server's own. A name with nothing on one
/// side of the joiner is not a qualification.
pub(crate) fn split(name: &str) -> (&str, Option<&str>) {
    match name.rsplit_once(JOINER) {
        Some((row, client)) if !row.is_empty() && !client.is_empty() => (row, Some(client)),
        _ => (name, None),
    }
}

/// The client `name` is on, or `None` for the server's own row.
pub(crate) fn of(name: &str) -> Option<&str> {
    split(name).1
}

/// The row `name` is, as its machine spells it.
pub(crate) fn row(name: &str) -> &str {
    split(name).0
}

/// A machine's word on the glass: the client's name, or [`SERVER`].
pub(crate) fn word(machine: Option<&str>) -> &str {
    machine.unwrap_or(SERVER)
}

impl Model {
    /// Every provider name this seat holds: the table's, then the worker's
    /// own where the table does not name it (bl-67aa — a worker bound to a
    /// preset nobody named is on no table row, and the lists still say where
    /// it stands).
    fn provider_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .providers
            .iter()
            .flatten()
            .map(|row| row.name.clone())
            .collect();
        if let Some(worker) = self.worker()
            && !names.contains(&worker.provider)
        {
            names.insert(0, worker.provider.clone());
        }
        names
    }

    /// **The machines**: the server, always and first, then each client a
    /// held name is on, in the table's order.
    pub(super) fn machine_rows(&self) -> Vec<Choice> {
        let at = self.worker().map(|w| of(&w.provider).map(str::to_owned));
        let mut machines: Vec<Option<String>> = vec![None];
        for name in self.provider_names() {
            let machine = of(&name).map(str::to_owned);
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
    /// picked by the table's own qualified name.
    pub(super) fn provider_rows(&self, machine: Option<&str>) -> Vec<Choice> {
        let worker = self.worker().map(|w| w.provider.as_str());
        self.provider_names()
            .into_iter()
            .filter(|name| of(name) == machine)
            .map(|name| Choice {
                current: worker == Some(name.as_str()),
                text: row(&name).to_owned(),
                pick: Pick::Provider(name),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests;
