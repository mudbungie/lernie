//! **The chat pane**: one conversation, entry by entry.
//!
//! Every entry becomes rows of `(who said it, what was said)`, and the
//! projection is a **pure function** of the transcript and the live fold — so
//! what the pane shows is a value a test reads back, and the paint below it is
//! thin enough to have nothing of its own to get wrong.
//!
//! **Nothing is dropped, including what nothing could parse.** An entry the
//! engine could not read is surfaced as its raw bytes, and an entry of a kind
//! this build does not know is surfaced as its own word beside them — the reply
//! vocabulary's rung 3, on the glass. A transcript that quietly skipped an
//! entry would be a conversation the operator reads as shorter than it was,
//! which is the one failure a transcript must not have.
//!
//! The one thing that is **not** a row is a half of a turn with nothing in it
//! — see [`half`], which is that rule's one home. An empty half is not
//! something the operator was not shown; it is something that was never said.

/// What a machine's answer hides when it is folded, and the counts that say so.
pub mod fold;
/// Every entry of a transcript as the rows this pane paints.
pub mod rows;
/// Which conversation this is: the header over the transcript.
pub mod subject;

pub use fold::Fold;
pub use rows::{Row, rows};

/// **The word this pane wears**, and the name of the column it is (bl-dfda).
/// It is painted by `crate::ui::shell` — above the pane in the broad shape, on
/// the navigation bar in the narrow one — because a column's name has one home
/// and which one it is depends on the shape.
pub const HEADING: &str = "conversation";

/// What the pane says with no conversation selected.
pub const NO_CONVERSATION: &str = "pick a conversation";
/// The name the live tail wears — no file backs it.
pub const LIVE: &str = "«live»";

/// Paint the pane. **The heading is the shell's** — see [`HEADING`].
///
/// **The pane is anchored to the TAIL, and the anchoring is a scroll state per
/// conversation rather than a gesture** (bl-83ae). Two facts do the whole of
/// it and neither is a flag on the model.
///
/// The scroll state is salted with the conversation's own id, so *where I am
/// in this transcript* is a property of the transcript rather than of the
/// pane: selecting a conversation for the first time meets a fresh state,
/// egui's fresh state is stuck-to-end, and the tail is what lands on the glass.
/// A conversation scrolled up in and come back to is where it was left, which
/// is the same rule read a second time.
///
/// [`egui::ScrollArea::stick_to_bottom`] is the following: while the offset is
/// at the end the pane rides every append — the follow lane's write cadence
/// included — and the first scroll away takes the stickiness off, because
/// egui re-derives it from *is the offset at the end* on every frame. So
/// scrolling back down starts it following again, with no control to press and
/// nothing on the model to get out of step. Until this the pane opened at
/// message 001 and stayed there: a live conversation's answer arrived thirty
/// screens below the fold, which is the whole middle of the window unusable
/// for the two things it is opened for.
pub fn render(ui: &mut egui::Ui, model: &crate::ui::Model) {
    let Some(conversation) = model.conversation.as_ref() else {
        ui.label(NO_CONVERSATION);
        return;
    };
    // **Which conversation this is, above the transcript** (bl-7b03). The
    // column's NAME is still the shell's — *conversation*, painted over the
    // pane in the broad shape and on the bar in the narrow one — and this is
    // its SUBJECT, which is content and belongs in the pane. It stands outside
    // the scrolled region for `crate::ui::roster`'s reason: it is the one thing
    // here that is always true of what is being read.
    subject::render(ui, model);
    egui::ScrollArea::vertical()
        .id_salt(conversation)
        .stick_to_bottom(true)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (at, row) in rows(&model.transcript, model.live.as_ref())
                .into_iter()
                .enumerate()
            {
                ui.separator();
                ui.strong(&row.who);
                match &row.fold {
                    Some(fold) => folded(ui, at, &row, fold),
                    None => {
                        ui.label(&row.said);
                    }
                }
            }
        });
}

/// **A machine's answer, folded**: its first few lines, and one control
/// carrying the act and the size of what is hidden (bl-90d0).
///
/// **Whether one row is open is the TOOLKIT's memory, keyed on the row**, for
/// [`render`]'s own reason one noun over: it is a per-row view state that no
/// other fact can be asked for and that no gesture on the wire corresponds to,
/// so putting it on the model would be plumbing a click through a snapshot to
/// get back to the widget that took it. The key is the row's place in the
/// transcript, under the id the scroll area already salted with the
/// conversation, so two conversations' fifth rows are two rows.
///
/// Open, the head stands down and the whole answer takes its place — the head
/// is a PREFIX of it, so painting both would paint the first six lines twice.
fn folded(ui: &mut egui::Ui, at: usize, row: &Row, fold: &Fold) {
    let id = ui.make_persistent_id(at);
    let mut open = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
    if ui
        .button(if open { fold::UNFOLD } else { &fold.whole })
        .clicked()
    {
        open = !open;
        ui.data_mut(|d| d.insert_temp(id, open));
    }
    ui.label(if open { &row.said } else { &fold.head });
}

#[cfg(test)]
mod tests;
