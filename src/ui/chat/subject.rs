//! **Which conversation this is**, over the transcript it is a header for
//! (bl-7b03).
//!
//! The pane carried the word *conversation* and nothing else — on every
//! conversation, in every state. A reader scrolling a transcript could not tell
//! whether what they were reading was still running, what it had cost, or which
//! model had answered; the facts existed, typed and rendered, on the records
//! pane two clicks away.
//!
//! **The column's NAME is still the shell's** (bl-dfda): *conversation* is what
//! the layout paints above the pane and what the narrow shape's bar carries,
//! and a second node wearing it would be two things to tell apart. What is here
//! is not a name but CONTENT — the subject the rest of the pane is about, which
//! is exactly where the records pane puts its own header (§4.32).
//!
//! **Every line is the engine's**, composed by `crate::ui::records::header` and
//! read from there rather than re-derived: the spend is a figure the engine
//! summed, the model is the one its context reading names, the failure clause
//! is the provider's own sentence. Two surfaces reading one answer is one home;
//! two surfaces composing one sentence is two.

use crate::reply::agent::Agent;
use crate::reply::transcript::{EntryKind, Transcript};
use crate::ui::records::header;

/// **What the pane says over a conversation nobody has been answered about
/// yet** — the records pane's own sentence, because it is the same absence.
pub use header::NOT_ANSWERED;

/// **The identity line**: what it is called, how it is resting, and — where
/// the costing line below does not already say it — which model answered.
///
/// The tip the records pane's own `named` carries is left out on purpose: a
/// branch oid is what a `git show` outside this seat takes, and the question
/// this line answers is *am I reading something that is still running*.
pub fn named(row: &Agent, transcript: &Transcript) -> String {
    let said = format!("{} — {}", row.display, header::resting(row));
    match answered_by(row, transcript) {
        Some(model) => format!("{said} — {model}"),
        None => said,
    }
}

/// **Which model answered, where nothing else on this header says so.**
///
/// Two places can answer and they answer different questions. The engine's
/// context reading names the model it is holding a window open for, which is
/// the one the NEXT turn will use — the better answer, and the one
/// [`costing`] already carries. But a reading exists only while there is one
/// to take, and a **quiescent** conversation has none: that is the state the
/// question is actually asked in, and it is why *nothing in the window says
/// which model answered* was true of every conversation that had finished.
///
/// So the transcript's own last model turn answers when the engine's reading
/// does not. It is a fact this seat already holds, about a turn that
/// happened, and never a prediction about the next one — which is also why it
/// stands down the moment the engine has a reading of its own, rather than
/// being joined beside it.
fn answered_by(row: &Agent, transcript: &Transcript) -> Option<String> {
    if row.context.is_some() {
        return None;
    }
    transcript
        .entries
        .iter()
        .rev()
        .find_map(|entry| match &entry.kind {
            EntryKind::Model { model_id, .. } => Some(model_id.clone()),
            _ => None,
        })
}

/// **The costing line**: what it has spent, and which model it is on.
///
/// One call rather than two facts joined here, because the engine states them
/// together and the joining is already stated once (`header::costing`).
pub fn costing(row: &Agent) -> String {
    header::costing(row)
}

/// Paint the header, or the sentence for a conversation nobody has answered
/// about yet.
pub fn render(ui: &mut egui::Ui, model: &crate::ui::Model) {
    let weak = crate::ui::theme::tone_ink(&crate::reply::convs::Tone::Weak);
    let Some(row) = model.records.agent.as_ref() else {
        ui.colored_label(weak, NOT_ANSWERED);
        return;
    };
    ui.strong(named(row, &model.transcript));
    // **A conversation that died on a bad model id must not look like one that
    // finished**, which is the ball's own sentence: the provider's clause is
    // the one fact that tells the two apart, and it goes above the costing
    // because it is why there is no more of it.
    if let Some(failure) = &row.failure {
        ui.colored_label(crate::ui::theme::NOTICE, failure);
    }
    ui.colored_label(weak, costing(row));
}

#[cfg(test)]
mod tests;
