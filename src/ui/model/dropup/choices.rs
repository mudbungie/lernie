//! **What an open dropup lists**: the rows this seat already holds, read at
//! once — the roster's workspaces, the providers read, one provider's models, the
//! levels — so the list stands before the refresh it posted has answered.

use super::{Choice, Model, Pick, Selector};

/// **The priority lane's word**, on the list and on the rule alike.
pub(crate) fn lane(on: bool) -> &'static str {
    if on { "on" } else { crate::verbs::tuning::OFF }
}

impl Model {
    /// **The open dropup's rows**, or none where nothing is open.
    pub fn choices(&self) -> Vec<Choice> {
        let Some(open) = &self.dropup else {
            return Vec::new();
        };
        let worker = self.worker();
        match open.selector {
            Selector::Workspace => self.workspaces(),
            // **Every row of the table is a choice** (bl-67aa; it filtered
            // bare rows off under bl-526d): the providers read is brazen's
            // routing table, so a preset nobody named and nobody signed into
            // is not in it at all. **The row in force is listed whether or
            // not the table names it** — a worker bound to such a preset is
            // on no table row — so the list still says where the worker
            // stands.
            Selector::Provider => {
                let mut names: Vec<String> = self
                    .providers
                    .iter()
                    .flatten()
                    .map(|row| row.name.clone())
                    .collect();
                if let Some(w) = worker
                    && !names.contains(&w.provider)
                {
                    names.insert(0, w.provider.clone());
                }
                names
                    .into_iter()
                    .map(|name| Choice {
                        current: worker.is_some_and(|w| w.provider == name),
                        text: name.clone(),
                        pick: Pick::Provider(name),
                    })
                    .collect()
            }
            Selector::Model => {
                let provider = open.provider.clone().unwrap_or_default();
                let mut ids = self.offered_by(&provider);
                // **What is in force is listed whether or not the listing
                // has come back**, so the list says where it stands now.
                if let Some(w) = worker.filter(|w| w.provider == provider)
                    && !ids.contains(&w.model)
                {
                    ids.insert(0, w.model.clone());
                }
                ids.into_iter()
                    .map(|id| Choice {
                        current: worker.is_some_and(|w| w.provider == provider && w.model == id),
                        text: id.clone(),
                        pick: Pick::Model(id),
                    })
                    .collect()
            }
            Selector::Effort => crate::verbs::tuning::levels()
                .into_iter()
                .map(|level| Choice {
                    text: crate::verbs::tuning::word(level.as_ref()),
                    current: worker.is_some_and(|w| w.effort == level),
                    pick: Pick::Effort(level),
                })
                .collect(),
            Selector::Priority => [true, false]
                .into_iter()
                .map(|on| Choice {
                    text: lane(on).to_owned(),
                    current: worker.is_some_and(|w| w.priority == on),
                    pick: Pick::Priority(on),
                })
                .collect(),
        }
    }

    /// **Every workspace this seat can address**, in the roster's order — named
    /// with its server where the seat reaches more than one — and last,
    /// `+ new workspace` under this box's own server where it is held
    /// (DESIGN §4.44: the switcher's row, beside the choice it belongs to).
    fn workspaces(&self) -> Vec<Choice> {
        let many = self.roster.len() > 1;
        let found = self
            .roster
            .iter()
            .find(|chunk| {
                chunk.channel.name == crate::seat::OWN
                    && !matches!(chunk.held, crate::ui::Held::Unheld(_))
            })
            .map(|chunk| Choice {
                text: crate::ui::roster::server::FOUND.to_owned(),
                current: false,
                pick: Pick::Found(chunk.channel.name.clone()),
            });
        self.roster
            .iter()
            .flat_map(|chunk| {
                chunk.workspaces.iter().filter_map(move |row| {
                    let address = chunk.channel.address(row)?;
                    let aim = super::Aim {
                        channel: chunk.channel.name.clone(),
                        address,
                    };
                    Some(Choice {
                        text: if many {
                            format!("{} · {}", row.workspace, chunk.channel.name)
                        } else {
                            row.workspace.clone()
                        },
                        current: self.aim.as_ref() == Some(&aim),
                        pick: Pick::Workspace(aim),
                    })
                })
            })
            .chain(found)
            .collect()
    }
}
