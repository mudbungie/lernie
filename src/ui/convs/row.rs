//! **One conversation on the glass** (`docs/STYLE.md` §5, *a threaded row*):
//! the row itself, the lines hung under it, the control that opens its
//! subtree, and the connectors that say what it hangs from.
//!
//! Split out of [`super`] at the design-time budget on the seam the list
//! already has: there is the LIST — the two emptinesses, the rail question and
//! the pure functions a row's words are made of — and here is what one row
//! puts on the glass. The first changes when a wall answers differently; the
//! second when the visual language does.
//!
//! **Nothing here draws its own row.** `theme::paint::row` is the anatomy and
//! this file is a caller of it, so the selection tint, the state rule and the
//! elision are one implementation rather than this pane's opinion of them.
//! The fill this file used to paint under its own label — a reserved
//! `Shape::Noop` set after the run, so the selected row was not a solid bar of
//! selection ink with its name invisible under it (bl-dc07) — is gone with the
//! widget that made it necessary.

use crate::reply::convs::ConvRow;
use crate::ui::{Aim, Model, theme};

use theme::paint;

/// How far one level of descent moves a row to the right, in points.
pub(crate) const STEP: f32 = 16.0;

/// Past this the indent says nothing more: a list deeper than eight is
/// unreadable at any width, and the label is what the extra width would have
/// cost.
const DEEPEST: u64 = 8;

/// The word on the control while a subtree is folded — see [`subtree`].
pub const SHOW: &str = "show";
/// And the one it wears while it is open.
pub const HIDE: &str = "hide";

/// **One row, and everything that hangs off it.**
///
/// `rails` is the ancestor levels whose thread continues past this row
/// ([`super::continues`]), which is a question about the LIST and so is
/// answered by the list — this file only draws what it is told.
pub(super) fn conversation(
    ui: &mut egui::Ui,
    model: &mut Model,
    aim: &Aim,
    row: &ConvRow,
    rails: &[u64],
    reveal: bool,
) {
    let selected = model.conversation.as_ref() == Some(&row.root_id);
    let state = super::rule(row);
    // **The state glyph leads, in its own ink** (§4.42 item 7), and the run
    // is the headline whole (`super::headline`).
    let (glyph, ink) = super::mark(row);
    // **The row is the whole block** (DESIGN §4.44 item 5): its headline and
    // the lines hung under it — the hue's own words, then the preview — are
    // one seat, so the tint fills what a click takes.
    let tone = theme::tone_ink(&row.tone);
    let beneath = [
        row.failure.clone().map(|failure| (failure, tone)),
        Some(row.preview.clone())
            .filter(|preview| !preview.is_empty())
            .map(|preview| (preview, theme::INK_WEAK)),
    ]
    .into_iter()
    .flatten()
    .map(|(said, ink)| (said, ink, egui::TextStyle::Body))
    .collect();
    let seat = paint::Row {
        mark: (glyph, ink),
        words: super::words(row),
        ink: tone,
        state,
        chosen: selected,
        indent: indent(row.depth),
        aside: None,
        beneath,
    }
    .paint(ui);
    // **Two reads hang off this one gesture** (`crate::ui::act`): selecting a
    // conversation is what makes this seat read its transcript and open the
    // follow lane on it, and neither has a control of its own.
    crate::ui::act::tag(
        &seat,
        &[crate::verbs::TRANSCRIPT.word, crate::verbs::FOLLOW.word],
    );
    if selected && reveal {
        seat.scroll_to_me(None);
    }
    if seat.clicked() {
        model.select(&row.root_id.clone());
        model.chose();
    }
    // **The row's own acts hang off the row** (bl-dbc9), on the gesture egui
    // synthesizes from a right-click and from a touch long-press alike. It is
    // the same response the click above is taken from, so there is no second
    // answer to *which conversation is this about*.
    super::menu::show(&seat, model, aim, row);
    subtree(ui, model, row);
    connectors(ui, row, rails, seat.rect);
}

/// **The control that opens what hangs under a conversation** (bl-00f5), on
/// the rows that have anything under them and on no others.
///
/// It names the act and how much of it there is — the pin pair's rule (§4.25)
/// and the tool fold's (§4.11): two words rather than one that toggles, so a
/// reader never has to work out which way it will go. The count is the
/// engine's `members` less this row itself.
///
/// **It is a row and it stands UNDER the name**, one step deeper — it is a
/// thing that hangs under the conversation, so it reads where the things that
/// hang under it will appear. Before the row anatomy it was a `small_button`
/// wedged in front of the name, which put a box on the glass (§1 rule 2) and
/// pushed every headline right by a control that most rows do not have.
fn subtree(ui: &mut egui::Ui, model: &mut Model, row: &ConvRow) {
    let Some(under) = row.members.checked_sub(1).filter(|under| *under > 0) else {
        return;
    };
    let word = if model.subtree_open(&row.root_id) {
        format!("{HIDE} {under}")
    } else {
        format!("{SHOW} {under}")
    };
    let seat = paint::row(
        ui,
        &word,
        theme::INK_WEAK,
        None,
        false,
        indent(row.depth) + STEP,
    );
    if seat.clicked() {
        model.toggle_subtree(&row.root_id.clone());
    }
}

/// **The threading connectors** — the phone's L-shaped idiom (yog-android
/// bl-4d17), in faint ink.
///
/// A rail stands at every ancestor level whose thread continues past this row
/// and spans everything the row put on the glass, so a reader can follow a
/// branch down through the members of the branches inside it. The elbow is
/// this row's own: down from its top to its middle, then across into its
/// words — the one line that says *this hangs from that*.
///
/// **They are painted last, so they are painted ON TOP.** A painter appends to
/// its layer, and a connector under the row's hover tint is a connector that
/// vanishes exactly when the pointer is on the row it explains.
fn connectors(ui: &egui::Ui, row: &ConvRow, rails: &[u64], seat: egui::Rect) {
    let bottom = ui.cursor().min.y;
    for level in rails {
        paint::rail(ui, at_level(seat.min.x, *level), seat.min.y, bottom);
    }
    if row.depth > 0 {
        paint::elbow(
            ui,
            at_level(seat.min.x, row.depth),
            seat.min.y,
            seat.min.y + theme::ROW / 2.0,
            seat.min.x + theme::RULE + indent(row.depth) + theme::space::M,
        );
    }
}

/// Where level `k`'s connector stands: half a step into the level it belongs
/// to, so the elbow leaves the parent's column and meets its own row's words.
fn at_level(left: f32, level: u64) -> f32 {
    left + indent(level.saturating_sub(1)) + STEP / 2.0
}

/// How far a row stands from the pane's left edge, in points: one [`STEP`]
/// under the wall it hangs from (DESIGN §4.39 — a conversation is a level
/// below its workspace, and the fold is what made that a thing the glass can
/// show), plus one more per level of descent inside the thread.
///
/// Added rather than multiplied, over a **bounded** count: there is no cast
/// from the wire's own width to a screen coordinate, so there is no truncation
/// to suppress a lint about.
pub(crate) fn indent(depth: u64) -> f32 {
    (0..depth.min(DEEPEST)).fold(STEP, |at, _| at + STEP)
}
