//! **The decision queue between frames** (bl-f0ef): open or not, what each
//! channel last said was asking, and the two acts its rows spend.
//!
//! # The first pane whose subject is not a focus
//!
//! The tuning pane is about the aimed workspace and the records pane about the
//! selected conversation, so both are retired when their subject moves. This
//! one is about **everything**: `attention` names no workspace, so its subject
//! is every channel this box holds and no aim or selection can invalidate it.
//! It therefore survives [`Model::aim_at`] and [`Model::select`] — which is not
//! an omission but the same rule those two keep, read on a pane whose subject
//! nothing on the glass can move.
//!
//! # A union across channels, replaced one channel at a time
//!
//! The read fans (`crate::offframe::asker`), so one pass is one frame per
//! channel and each answer is that server's whole queue. So an answer replaces
//! **its own channel's** [`Asking`] and leaves every other standing — REMOTE
//! §8.2's *"a refusal is one entry's, never the set's"*, the same shape
//! [`Model::seat`](super::Model) keeps for the roster.
//!
//! # A row is addressed off the ROSTER, within the channel it came down
//!
//! A queue row names its workspace as **its host** names it, which is not what
//! a gesture from this box must carry when an entry renames (§8.2). The
//! resolution already has one home in this window — a roster row and
//! `crate::ui::Channel::address` — so [`Model::workspace`] asks it there. **But
//! only in the roster chunk of the channel the row's section came down**
//! (bl-666f): a workspace's name is per server, and `home` is every fresh server's
//! bootstrap workspace, so a walk of the whole roster for the first workspace of that
//! name aimed a second server's rows at the first — *go to it* opened a
//! conversation that did not exist there, and `seen` and `answer` went down
//! the wrong channel. The section's stamp is the channel that answered
//! (`crate::offframe::poster`, bl-c70d), so it is a fact to resolve within,
//! and every door below takes it beside the row. A row for a workspace this seat
//! holds no name for on that channel is honestly unaddressable, and paints as
//! such, rather than being aimed by a guess at a different workspace.

use super::{Aim, Model};
use crate::reply::queue::QueueRow;
use crate::ui::Channel;

/// **One channel's answer to *what is asking*** — the rows, and the channel
/// they came down as the client's own stamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asking {
    pub channel: Channel,
    pub rows: Vec<QueueRow>,
}

/// **One SERVER's queue as the pane paints it** (bl-52d2): its header — every
/// name it is held under, `super::Server::header` — and each asking row once,
/// beside the channel a gesture at it goes down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub header: String,
    pub rows: Vec<(Channel, QueueRow)>,
}

impl Model {
    /// **What the operator calls the conversation `seen` answered**
    /// (bl-c887): the display name its queue row carries — read before the
    /// receipt refiles the queue — or the aimed workspace's row for it, and the
    /// internal id only where neither is at hand. The receipt names it by id,
    /// which no other surface paints.
    pub(super) fn seen_name(
        &self,
        channel: &Channel,
        acked: &crate::reply::queue::Acknowledged,
    ) -> String {
        let aimed = self
            .aim
            .as_ref()
            .is_some_and(|aim| aim.address == acked.workspace);
        self.waiting
            .iter()
            .filter(|held| held.channel.name == channel.name)
            .flat_map(|held| held.rows.iter())
            .chain(&acked.rows)
            .find(|row| row.workspace == acked.workspace && row.agent == acked.agent)
            .map(|row| row.display.clone())
            .or_else(|| {
                self.rows()
                    .into_iter()
                    .find(|row| aimed && row.root_id == acked.agent)
                    .map(|row| row.display)
            })
            .unwrap_or_else(|| format!("{}/{}", acked.workspace, acked.agent))
    }

    /// **Open the queue pane.** It takes no subject, so nothing gates it: the
    /// question *what is waiting on me* is answerable from an unaimed,
    /// unselected seat, and that is the seat most likely to be asking it.
    pub fn begin_queue(&mut self) {
        self.stand(super::Listing::Queue);
    }

    /// **Close it.** The rows stay, for the reason the roles and the records
    /// do: the next open is about the same queue, and the standing read
    /// replaces them anyway.
    pub fn close_queue(&mut self) {
        self.put_down(super::Listing::Queue);
    }

    /// File one channel's queue, replacing what that channel last said and
    /// leaving every other channel standing.
    pub(super) fn asking(&mut self, channel: &Channel, rows: Vec<QueueRow>) {
        let answered = Asking {
            channel: channel.clone(),
            rows,
        };
        match self
            .waiting
            .iter_mut()
            .find(|held| held.channel.name == channel.name)
        {
            Some(held) => *held = answered,
            None => self.waiting.push(answered),
        }
    }

    /// **The queue folded by server** (bl-52d2; bl-13b9's fold, one pane
    /// over). The fan asks every channel, so two channels dialling one
    /// address each answer that server's whole queue; the sections are keyed
    /// by the server and a row is kept once by `(workspace, agent)` — the
    /// host's own spelling, which both answers carry alike — beside the first
    /// channel that can address its workspace. A query over [`Model::waiting`],
    /// never a second store: the answers stay one per channel because each is
    /// replaced by its own channel's next one.
    pub fn sections(&self) -> Vec<Section> {
        let servers = self.folded();
        let mut out: Vec<(String, Section)> = Vec::new();
        for asked in &self.waiting {
            let key = self.server_of(&asked.channel.name);
            let at = out
                .iter()
                .position(|(held, _)| *held == key)
                .unwrap_or_else(|| {
                    let header = servers
                        .iter()
                        .find(|server| server.holds(&asked.channel.name))
                        .map_or_else(
                            || crate::ui::roster::header(&asked.channel),
                            super::Server::header,
                        );
                    out.push((
                        key,
                        Section {
                            header,
                            rows: Vec::new(),
                        },
                    ));
                    out.len() - 1
                });
            if let Some((_, section)) = out.get_mut(at) {
                self.keep(&mut section.rows, asked);
            }
        }
        out.into_iter().map(|(_, section)| section).collect()
    }

    /// **File one channel's rows into its server's section**: a row not yet
    /// held is kept, and one held down a channel that cannot address its workspace
    /// moves to this one.
    fn keep(&self, rows: &mut Vec<(Channel, QueueRow)>, asked: &Asking) {
        for row in &asked.rows {
            let seat = (asked.channel.clone(), row.clone());
            match rows
                .iter_mut()
                .find(|(_, held)| held.workspace == row.workspace && held.agent == row.agent)
            {
                None => rows.push(seat),
                Some(kept) if self.workspace(&kept.0, &row.workspace).is_none() => *kept = seat,
                Some(_) => {}
            }
        }
    }

    /// **The aim that reaches the workspace a queue row names on the channel it
    /// came down**, or `None` where this seat holds no name for it there.
    ///
    /// It reads the roster because the roster is where this window's half of
    /// §8.2's mapping lives: a workspace's row plus the channel it came down is
    /// exactly what `Channel::address` answers, and asking it here means the
    /// queue cannot hold a second opinion about how a workspace is addressed. It
    /// reads ONE chunk of it — `channel`'s — because a workspace's name is only
    /// unique within one server (bl-666f).
    pub fn workspace(&self, channel: &Channel, workspace: &str) -> Option<Aim> {
        let chunk = self
            .roster
            .iter()
            .find(|chunk| chunk.channel.name == channel.name)?;
        chunk
            .workspaces
            .iter()
            .find(|row| row.workspace == workspace)
            .and_then(|row| chunk.channel.address(row))
            .map(|address| Aim {
                channel: chunk.channel.name.clone(),
                address,
            })
    }

    /// **Answer a row's place in the queue**, or do nothing where this seat
    /// cannot address the workspace it names. The gate is the address rather than a
    /// separate check: a gesture with nowhere to go is a gesture nobody can
    /// compose, and the pane paints that row's own sentence instead of a
    /// control.
    pub fn post_seen(&mut self, channel: &Channel, row: &QueueRow) {
        if let Some(aim) = self.workspace(channel, &row.workspace) {
            self.outbox.push(super::Posted::act(crate::verbs::seen(
                aim.address,
                row.agent.clone(),
            )));
        }
    }

    /// **Answer the invocation parked at a row's conversation**, or do nothing
    /// where this seat cannot address the workspace it names — [`Self::post_seen`]'s
    /// gate, for its reason exactly.
    ///
    /// *Which* call is answered is read at the far end off the conversation's
    /// own hold mark at fire time, so this end names no invocation and cannot
    /// spend one that is no longer parked (`crate::verbs::capability`). What
    /// this end does say is the two words: the verdict, and **how far it
    /// stands** — stated on every gesture, `call` included, because the wire
    /// requires it in both directions.
    pub fn post_answer(&mut self, channel: &Channel, row: &QueueRow, verdict: &str, scope: &str) {
        if let Some(aim) = self.workspace(channel, &row.workspace) {
            self.outbox.push(super::Posted::act(crate::verbs::answer(
                aim.address,
                row.agent.clone(),
                verdict.to_owned(),
                Some(scope.to_owned()),
            )));
        }
    }

    /// **Go to the conversation a row is about**: aim at its workspace, select it,
    /// and stand the pane down.
    ///
    /// It crosses no wire of its own — an aim and a selection are views, which
    /// yog's `docs/PARITY.md` §2 puts out of the parity contract — and it
    /// carries no `act:` token for exactly that reason. What it spends is the
    /// two doors a click on the roster and a click on the list already spend,
    /// so a queue row reaches the conversation by the same gestures rather than
    /// by a third spelling of *look at this one*.
    pub fn go_to(&mut self, channel: &Channel, row: &QueueRow) {
        let Some(aim) = self.workspace(channel, &row.workspace) else {
            return;
        };
        self.aim_at(&aim.channel, &aim.address);
        self.select(&row.agent.clone());
        self.close_queue();
    }
}

#[cfg(test)]
mod tests;
