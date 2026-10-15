//! **What this seat sent, as rows** (DESIGN §4.45 item 4; bl-ca3d): one
//! operator row per message addressed to the selected conversation that the
//! committed transcript does not yet carry, after the committed rows and the
//! live fold.
//!
//! **The projection paints what stands and retires nothing.** Which records
//! stand is decided where each fact arrives — the receipt, the read, the
//! refusal (`crate::ui::model::Sent`) — so this is still a pure function of
//! the model, and the one thing it adds to [`super::rows`] is the end of the
//! list. It reads the whole model rather than a third argument because the
//! records are keyed by the selection, which the transcript alone cannot say.

use super::{Row, rows};
use crate::ui::Model;
use crate::ui::theme::Speaker;

impl Row {
    /// **A message this seat sent**: the operator's own row, headed as the
    /// engine's will be (`super::OPERATOR`'s first word is the sender litany
    /// writes a deposit from the seat with), so the row the engine's replaces
    /// it with reads the same — and pending until the receipt lands.
    fn sent(sent: &crate::ui::Sent) -> Self {
        Self {
            pending: !sent.acked,
            ..Self::plain(
                super::OPERATOR[0].to_owned(),
                sent.words.clone(),
                Speaker::Operator,
            )
        }
    }
}

/// **The whole pane as data**: the committed rows with the live fold in its
/// place ([`rows`]), then what this seat sent here and the read has not
/// carried back ([`Model::sent_here`]).
pub fn whole(model: &Model) -> Vec<Row> {
    let mut out = rows(&model.transcript, model.live.as_ref());
    out.extend(model.sent_here().map(Row::sent));
    out
}
