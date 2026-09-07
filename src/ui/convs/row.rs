//! **One conversation on the glass** (`docs/STYLE.md` §5, *a threaded row*):
//! the row itself, the lines hung under it, the control that opens its
//! subtree, and the connectors that say what it hangs from.
//!
//! Split out of [`super`] at the design-time budget on the seam the pane
//! already has: there is the LIST — four emptinesses, a scroll area, and the
//! two pure functions a row's words are made of — and here is what one row
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
pub(super) const STEP: f32 = 16.0;

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
    // **Asking outranks the state word.** A row that wants the operator wears
    // the attention accent whatever its driver is doing, because the whole of
    // §2's *the eye lands on green* is that nothing else is green — a row
    // asking while it streams would otherwise be purple and lost. Where
    // nothing is waiting the rule is the state's, and a state this build does
    // not know gets none rather than one it is not (`theme::state_of`).
    let state = if row.attention > 0 {
        Some(theme::accent(theme::State::Attention))
    } else {
        theme::state_of(&row.state).map(theme::accent)
    };
    let seat = paint::row(
        ui,
        &super::headline(row),
        theme::tone_ink(&row.tone),
        state,
        selected,
        indent(row.depth),
    );
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
    // **Tab and the arrows agree** (bl-2d6b), as on the roster's rows: a Tab
    // that lands here hands the arrows to the list.
    if seat.gained_focus() {
        model.focus = crate::ui::keys::Pane::Conversations;
    }
    if seat.clicked() {
        model.select(&row.root_id.clone());
    }
    // **The row's own acts hang off the row** (bl-dbc9), on the gesture egui
    // synthesizes from a right-click and from a touch long-press alike. It is
    // the same response the click above is taken from, so there is no second
    // answer to *which conversation is this about*.
    super::menu::show(&seat, model, aim, row);
    // **The hue's own words** (REMOTE §9.10), above the preview because the
    // preview is what was last said and this is why nothing more was: the row
    // reads top-down as label, reason, last words. The clause keeps the row's
    // own tone — the engine states the hue and the clause separately, and a
    // seat that inferred one would hold a second opinion about a reading it
    // did not take. The preview is a line after the first, so it is weak ink
    // (§5).
    if let Some(failure) = &row.failure {
        beneath(ui, row, failure, theme::tone_ink(&row.tone));
    }
    if !row.preview.is_empty() {
        beneath(ui, row, &row.preview, theme::INK_WEAK);
    }
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

/// A line hung under a row's headline, **aligned with the row's words** —
/// the row's indent plus the state rule plus `M`, which is exactly where
/// `theme::paint::row` lays its run. One function rather than two blocks that
/// must not drift: the second line of a row is a shape, and a second copy of
/// it is a second shape.
///
/// **It TRUNCATES, for bl-b3b2's reason one line down** (bl-fef8). A side
/// panel paints a frame sized to its own `max_width` and reserves the
/// CONTENT's right edge from the layout: an extending line pushes the second
/// past the first, and the strip between them is covered by no panel at all —
/// the ~150-point band of bare window surface the ball measured beside the
/// list. On the glass it ran flush into the next panel with no `…`, so a
/// reader could not tell the sentence continued.
fn beneath(ui: &mut egui::Ui, row: &ConvRow, said: &str, ink: egui::Color32) {
    ui.horizontal(|ui| {
        ui.add_space(indent(row.depth) + theme::RULE + theme::space::M);
        ui.add(egui::Label::new(egui::RichText::new(said).color(ink)).truncate());
    });
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
            seat.center().y,
            seat.min.x + theme::RULE + indent(row.depth) + theme::space::M,
        );
    }
}

/// Where level `k`'s connector stands: half a step into the level it belongs
/// to, so the elbow leaves the parent's column and meets its own row's words.
fn at_level(left: f32, level: u64) -> f32 {
    left + indent(level.saturating_sub(1)) + STEP / 2.0
}

/// How far a row hangs under its root, in points.
///
/// Added rather than multiplied, over a **bounded** count: there is no cast
/// from the wire's own width to a screen coordinate, so there is no truncation
/// to suppress a lint about.
fn indent(depth: u64) -> f32 {
    (0..depth.min(DEEPEST)).fold(0.0, |at, _| at + STEP)
}
