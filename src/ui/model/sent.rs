//! **A sent message between frames** (DESIGN §4.45 item 4; bl-ca3d): what
//! this seat composed and has not yet seen come back as the engine's own row.
//!
//! **The record is the ruling's own sentence** — *there should never be a
//! half-second where your message is nowhere*. The box clears on firing and
//! the read that carries the words lands at ask cadence, so between the two
//! the words would be in no pane at all; the record is what the transcript
//! paints over that gap (`crate::ui::chat::rows::whole`). It is a gesture
//! this seat made, [`Model::tabs`]'s class of fact — per run, never in the
//! place file, keyed by conversation so a tab change keeps each with its own
//! transcript.
//!
//! **Three endings, each decided where the fact arrives**, and none in the
//! projection, which paints what stands and mutates nothing: the receipt
//! ([`Model::acknowledged`]) changes the ink; the committed transcript
//! ([`Model::carried`]) retires the record the frame it holds the words;
//! a refusal, or an act that never left this box ([`Model::unsent`]), drops
//! the record and the words are the operator's again.

use super::{Model, Posted};
use crate::reply::transcript::{EntryKind, Transcript};
use crate::reply::{Read, Reply};

/// **One message this seat sent**, standing until the engine's row replaces
/// it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sent {
    /// The conversation it was addressed to: the one its row paints under.
    pub conversation: String,
    /// The words, verbatim — what the row paints and what the transcript's
    /// delivery is matched against.
    pub words: String,
    /// **How many entries the committed transcript held when it was
    /// composed.** The delivery that carries the words can only stand past
    /// this index, so an older deposit of the same words never retires it —
    /// save where no read had landed yet, when the count is zero and the
    /// match is as wide as the transcript (a cost bounded by one ask
    /// cadence and stated here).
    pub after: usize,
    /// Whether the act's receipt landed `ok`: the ink, and nothing else.
    pub acked: bool,
}

/// **The two deposits**: the acts whose words are the operator's own row on
/// the transcript, and the only acts a record is kept for.
pub(super) const DEPOSITS: [&str; 2] = [crate::verbs::MESSAGE.word, crate::verbs::INTERRUPT.word];

impl Model {
    /// **Compose one deposit and record it as sent**, one act — the one door
    /// the box (`crate::ui::composer::fire`) and the slash line
    /// (`crate::ui::slash::fire`) both go through, so there is no deposit on
    /// this window that is not on the glass the frame it is composed.
    ///
    /// `door` is `crate::verbs::message` or `crate::verbs::interrupt`: the
    /// two take the same three strings and differ only in the word, which is
    /// what a function pointer carries.
    pub fn deposit(
        &mut self,
        door: fn(String, String, String) -> serde_json::Value,
        workspace: String,
        agent: String,
        words: String,
    ) {
        self.sent.push(Sent {
            conversation: agent.clone(),
            words: words.clone(),
            after: self.transcript.entries.len(),
            acked: false,
        });
        self.outbox.push(Posted::act(door(workspace, agent, words)));
    }

    /// **The records addressed to the selected conversation**, oldest first:
    /// what the transcript paints after its committed rows.
    pub(crate) fn sent_here(&self) -> impl Iterator<Item = &Sent> {
        self.sent
            .iter()
            .filter(|sent| Some(sent.conversation.as_str()) == self.conversation.as_deref())
    }

    /// **A deposit's receipt** (`super::absorb`, the `message` and `interrupt`
    /// ops), answered to the oldest record not yet acknowledged — the acts
    /// cross one channel in the order they were posted, and a receipt names
    /// no conversation of its own. An `ok` outcome marks it acknowledged,
    /// which is the ink changing and nothing else: the row stays until the
    /// transcript carries the words, because a row retired on the receipt
    /// would be a message nowhere for exactly the half-second the ruling
    /// forbids. A refusal drops the record and the words go back in the box.
    /// Anything else — a failed run, a frame this seat cannot read — is the
    /// bar's sentence, and the row stands in the pending ink beside it.
    pub(super) fn acknowledged(&mut self, op: &str, read: &Read) {
        if !DEPOSITS.contains(&op) {
            return;
        }
        let Some(at) = self.sent.iter().position(|sent| !sent.acked) else {
            return;
        };
        match read {
            Read::Answer(Reply::Outcome(run)) if run.ok() => {
                if let Some(sent) = self.sent.get_mut(at) {
                    sent.acked = true;
                }
            }
            Read::Refusal(_) => {
                let sent = self.sent.remove(at);
                self.restore(sent.words);
            }
            _ => {}
        }
    }

    /// **An act that never left this box** ([`Model::acted`]): the record goes
    /// with it, because no receipt and no read will ever retire it, and the
    /// words are `refund`'s. One that crossed and was never answered keeps
    /// its row in the pending ink beside the in-doubt notice — the engine
    /// may have it, and a row that vanished would say it does not.
    pub(super) fn unsent(&mut self, op: &str, reach: &crate::channel::Reach) {
        if !DEPOSITS.contains(&op) || reach.crossed() {
            return;
        }
        if let Some(at) = self.sent.iter().position(|sent| !sent.acked) {
            self.sent.remove(at);
        }
    }

    /// **The committed transcript landed** (`super::absorb`): every record
    /// addressed to the selected conversation whose words it now carries —
    /// an operator delivery past the recorded length saying exactly them —
    /// is dropped the same frame, so the words are the engine's row and never
    /// two rows. Records for other conversations stand: the read is the
    /// selected one's.
    pub(super) fn carried(&mut self, transcript: Transcript) {
        let selected = self.conversation.clone();
        self.sent.retain(|sent| {
            selected.as_deref() != Some(sent.conversation.as_str()) || !carries(&transcript, sent)
        });
        self.transcript = transcript;
    }

    /// **Refused words go back in the box, ahead of whatever is typed.** The
    /// start's rule (`Model::refund`, bl-b180) gives words back only to an
    /// empty box, because a goal overwriting a newer draft loses the newer
    /// of the two; a refused message was already on the glass as a row, and
    /// the box is the only place left for it — so it goes in front, with a
    /// line break, and neither the refused words nor the draft is lost.
    fn restore(&mut self, words: String) {
        self.draft = if self.draft.trim().is_empty() {
            words
        } else {
            format!("{words}\n{}", self.draft)
        };
    }
}

/// Whether `transcript` holds `sent`'s words as an operator's delivery past
/// the recorded length.
fn carries(transcript: &Transcript, sent: &Sent) -> bool {
    transcript.entries.iter().skip(sent.after).any(|entry| {
        matches!(
            &entry.kind,
            EntryKind::Delivered { sender, body, .. }
                if crate::ui::chat::rows::OPERATOR.contains(&sender.as_str()) && *body == sent.words
        )
    })
}

#[cfg(test)]
mod tests;
