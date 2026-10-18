//! **One server, every name it is held under** (bl-13b9; DESIGN §4.39).
//!
//! The channel directory is the operator's, and it may lawfully name one
//! server twice — two entries terminating at one listener (§8.2), or a
//! workspace entry enrolled before this box became a server itself. Each is a
//! channel and each answers; painted one section a channel, the seat showed
//! one server as two servers and counted its waiting twice. The window's unit
//! is the SERVER, so the channels are folded by the address they dial before
//! anything is counted or painted, and every name stays beside the one row.
//!
//! **It is a query over the roster and never a second roster** — the chunks
//! stay one per channel, because a reply is stamped with the channel it came
//! down and a gesture goes down the channel that can address its workspace.

use crate::reply::roster::WsRow;

use super::{Aim, Chunk, Held, Model};

/// **One server as the window shows it**: every channel this seat holds that
/// dials one address, the lead apart from the rest so that a server with no
/// channel cannot be written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Server {
    /// This box's own server where it is one of them — the channel that
    /// addresses every workspace by its own name — else the first held.
    pub lead: Chunk,
    /// Every other name it is held under, in the order held.
    pub also: Vec<Chunk>,
}

impl Server {
    /// The name the server's row goes by: its lead's.
    pub(crate) fn name(&self) -> &str {
        &self.lead.channel.name
    }

    /// Every channel it is held under, the lead first.
    pub(crate) fn chunks(&self) -> impl Iterator<Item = &Chunk> {
        std::iter::once(&self.lead).chain(&self.also)
    }

    /// Whether `channel` is one of the names this server is held under.
    pub fn holds(&self, channel: &str) -> bool {
        self.chunks().any(|chunk| chunk.channel.name == channel)
    }

    /// **Reached if any of its channels answered**; else the first that says
    /// why — it will not dial, or it refused the roster; else not heard from
    /// yet.
    pub fn held(&self) -> Held {
        let all = || self.chunks().map(|chunk| &chunk.held);
        all()
            .find(|held| **held == Held::Heard)
            .or_else(|| all().find(|held| **held != Held::Unheard))
            .unwrap_or(&self.lead.held)
            .clone()
    }

    /// **Each workspace once**, as the first channel to answer it said it.
    pub fn workspaces(&self) -> Vec<WsRow> {
        let mut out: Vec<WsRow> = Vec::new();
        for workspace in self.chunks().flat_map(|chunk| &chunk.workspaces) {
            if !out.iter().any(|held| held.workspace == workspace.workspace) {
                out.push(workspace.clone());
            }
        }
        out
    }

    /// **Where a gesture at `row` goes**: down the first of this server's
    /// channels that can address it, or nowhere.
    pub fn aim(&self, row: &WsRow) -> Option<Aim> {
        self.chunks().find_map(|chunk| {
            chunk.channel.address(row).map(|address| Aim {
                channel: chunk.channel.name.clone(),
                address,
            })
        })
    }

    /// **The row's words**: the lead's header, and every other name it is
    /// held under beside it.
    pub fn header(&self) -> String {
        let lead = crate::ui::roster::header(&self.lead.channel);
        if self.also.is_empty() {
            return lead;
        }
        let also: Vec<&str> = self
            .also
            .iter()
            .map(|chunk| chunk.channel.name.as_str())
            .collect();
        format!("{lead} — {ALSO} {}", also.join(", "))
    }
}

/// What precedes the other names one server is held under.
pub const ALSO: &str = "also held as";

impl Model {
    /// **The roster folded by address**, in the order its first channel was
    /// held. A channel that dials nothing this seat knows stands alone.
    pub fn folded(&self) -> Vec<Server> {
        let mut out: Vec<Server> = Vec::new();
        for chunk in &self.roster {
            let same = |server: &&mut Server| {
                chunk.channel.dials.is_some() && server.lead.channel.dials == chunk.channel.dials
            };
            match out.iter_mut().find(same) {
                // The box's own server leads: it addresses every workspace.
                Some(server) if chunk.channel.named_there.is_none() => {
                    let was = std::mem::replace(&mut server.lead, chunk.clone());
                    server.also.insert(0, was);
                }
                Some(server) => server.also.push(chunk.clone()),
                None => out.push(Server {
                    lead: chunk.clone(),
                    also: Vec::new(),
                }),
            }
        }
        out
    }

    /// **The server `channel` is one name of**, by its lead's name.
    pub fn lead_of(&self, channel: &str) -> Option<String> {
        self.folded()
            .into_iter()
            .find(|server| server.holds(channel))
            .map(|server| server.name().to_owned())
    }

    /// **The server a gesture down `channel` is a use of** — its lead's name,
    /// and a channel this seat no longer holds is its own.
    pub fn server_of(&self, channel: &str) -> String {
        self.lead_of(channel).unwrap_or_else(|| channel.to_owned())
    }
}

#[cfg(test)]
mod tests;
