//! **Assertion (d): every control lights under the pointer** (DESIGN §4.45
//! item 6).
//!
//! The ruling is that *state should always be immediately represented*, and
//! what the pointer is over is state the glass should show. A seat that
//! senses a click and paints nothing for it is the quietest way to break
//! that: the operator's hand is on a door and the door does not say so. The
//! audit routed every such seat through `theme::paint`'s door or the row
//! idiom; this is what keeps a later one from arriving bare.
//!
//! **The subject is each tagged control**, read off the same accessibility
//! tree the other three assertions walk and `super::parity` inventories —
//! so the set judged here is the set the parity gate already owns, and a
//! control that reaches the ledger reaches this. The pointer is put just
//! above the control — past egui's own reach, which counts a pointer within
//! `interact_radius` of a seat as on it — the frame settles, and the fills
//! inside the control's rectangle are read off the frame
//! (`crate::paint_probe::fills_of`, the one walk); then the pointer is put
//! at its centre and the same fills read again. **Lit means changed**: a row
//! turns its transparent fill into the wash, a button its rest fill into the
//! hover one, a link grows a fill it did not have. A control whose fills are
//! the same with the pointer on it as above it is bare.
//!
//! **Above, not absent, and on a clock of its own.** The bar opens to a
//! pointer near the window's left edge and shuts on a linger
//! (`crate::ui::bar`), so a bar row judged against a frame with no pointer
//! at all is judged against a different bar; a pointer straight above the
//! row is as far into the bar as the pointer on it, so the bar is the same
//! bar in both frames — which is also why the rest point is above and not
//! beside, where a step to the left of the first column would open the bar
//! under a control that is not in it. Each control begins with the pointer
//! gone and is then found under the pointer, because a window lays itself
//! out differently with a pointer in it: the bar opens, a scrolling pane
//! makes room for its bar, and a wrapped strip breaks on a different word.
//! And the linger and the ease run on egui's clock,
//! which is the wall's unless the input says otherwise — so every frame
//! here says otherwise, stepping the clock by more than the linger, and the
//! frame the fills are read off is the settled one whatever the machine was
//! doing.
//!
//! # What is not judged, and by which fact
//!
//! Each exclusion is a fact the tree or egui states about the control, not
//! a name on a list:
//!
//! - **A control egui does not report hovered** with the pointer at its
//!   centre. It is disabled, or scrolled past its clip, or covered by an
//!   overlay — in every case the pointer is not on it, and a wash under a
//!   pointer that is somewhere else would be the defect, not the fix.
//!   Where the control stands is read off egui with the pointer on it, so
//!   a control that moved when the bar opened or a pane made room for its
//!   scroll bar is aimed at where it moved to.
//! - **A selected control.** The chosen row wears `RAISED` and the brand
//!   rule, which outranks the wash (`theme::hover`'s own ordering): it is
//!   already the lit one, and the pointer arriving changes nothing it says.
//!   egui writes a selection into the tree as *toggled*, which is also what
//!   a checked box is — and a box's check is a value, not a place the
//!   operator is, so a check box is judged whichever way it is set.
//! - **A field that takes text.** Its answer to the pointer is the caret;
//!   a wash on a text box is a box that looks like a button.
//!
//! **The way from a node to egui's seat is the tag's own bridge**
//! (`crate::ui::act::seat_of`): AccessKit's id is the egui id's value, and
//! egui answers nothing about a number, so the tag records the seat it
//! wrote and this reads it back — where the control stands, and whether the
//! pointer on it is on it.

use crate::paint_probe::{Filled, fills_of};
use crate::ui::{Model, act};
use egui_kittest::Harness;
use egui_kittest::kittest::{Queryable, by};

/// One tagged control as the tree offers it, held apart from the tree so the
/// harness can be driven while the list is walked.
struct Control {
    node: u64,
    tags: String,
    label: String,
    role: String,
    selected: bool,
}

/// The fills a frame put inside `rect`, in the order they were painted.
fn inside(fills: &[Filled], rect: egui::Rect) -> Vec<Filled> {
    fills
        .iter()
        .filter(|(fill, _)| fill.intersects(rect))
        .copied()
        .collect()
}

/// **Whether a role takes text**, by the tree's own name for it.
fn takes_text(role: &str) -> bool {
    role.ends_with("TextInput")
}

/// **Whether a control is the chosen one**: toggled on, and not a check box.
/// Both are read by their names because the types they come in belong to a
/// crate this one does not declare (`super::clipped::fault`'s reasoning).
fn selected(role: &str, toggled: &str) -> bool {
    toggled == "Some(True)" && role != "CheckBox"
}

/// **Every tagged control**, read in one pass over the settled tree.
fn controls(harness: &Harness<'_, Model>) -> Vec<Control> {
    harness
        .query_all(by())
        .filter(|node| !node.is_hidden())
        .filter_map(|node| {
            let tags = node.author_id()?.to_owned();
            Some(Control {
                node: node.id().0,
                tags,
                label: node.label().or_else(|| node.value()).unwrap_or_default(),
                role: format!("{:?}", node.role()),
                selected: selected(
                    &format!("{:?}", node.role()),
                    &format!("{:?}", node.toggled()),
                ),
            })
        })
        .collect()
}

/// **The fills of the last settled frame**. The frame is read by path
/// rather than as a method because `rules/no-bare-fork.yml` reads
/// `.output()` on anything as a child's — and this is a frame, not a fork.
fn fills(harness: &Harness<'_, Model>) -> Vec<Filled> {
    fills_of(Harness::output(harness))
}

/// **Hand the frame one pointer event and settle it**, two passes on a
/// clock that steps past the bar's linger each pass — so what the frame
/// holds is what the pointer there holds, and never where an animation was.
fn settle(harness: &mut Harness<'_, Model>, event: egui::Event) {
    harness.input_mut().events.push(event);
    for _ in 0..2 {
        let input = harness.input_mut();
        input.time = Some(input.time.unwrap_or(1_000.0) + crate::ui::bar::LINGER + 1.0);
        harness.step();
    }
}

/// **Every tagged control, hovered and judged.** The count is how many were
/// judged at all — a screen that judged none is a screen this said nothing
/// about, and the caller holds the matrix to a floor. An empty list is the
/// assertion holding.
pub(crate) fn complaints(at: &str, harness: &mut Harness<'_, Model>) -> (usize, Vec<String>) {
    let mut judged = 0;
    let mut out = Vec::new();
    for control in controls(harness) {
        if control.selected || takes_text(&control.role) {
            continue;
        }
        // **A tagged node has a seat, or the bridge is broken**: the tag wrote
        // both in the pass the tree was read off, so an absence here is not
        // a control to skip but the instrument gone quiet, said loudly.
        let id = act::seat_of(&harness.ctx, control.node).expect("a tagged node recorded its seat");
        settle(harness, egui::Event::PointerGone);
        let mut seat = harness
            .ctx
            .read_response(id)
            .expect("a tagged node is a seat egui holds");
        // **Where it stands is where it stands under the pointer**: the bar
        // opens to a pointer inside it and a scrolling pane makes room for
        // its bar once a pointer is over it, so a layout read with the
        // pointer gone is not the one the pointer on the control meets. Two
        // moves to its centre, re-reading after each, land on that one.
        for _ in 0..2 {
            settle(harness, egui::Event::PointerMoved(seat.rect.center()));
            seat = harness
                .ctx
                .read_response(id)
                .expect("a seat the pointer stands on is still held");
        }
        let clear = harness.ctx.style().interaction.interact_radius + crate::ui::theme::space::XS;
        let above = seat.rect.center_top() - egui::vec2(0.0, clear + 1.0);
        settle(harness, egui::Event::PointerMoved(above));
        let rest = fills(harness);
        settle(harness, egui::Event::PointerMoved(seat.rect.center()));
        let Some(under) = harness
            .ctx
            .read_response(id)
            .filter(egui::Response::hovered)
        else {
            continue;
        };
        judged += 1;
        let before = inside(&rest, under.rect);
        let after = inside(&fills(harness), under.rect);
        if before == after {
            out.push(format!(
                "{at}: the {} reading {:?} ({}) paints the same fills with the pointer on \
                 it as without — it senses a click and does not light",
                control.role, control.label, control.tags
            ));
        }
    }
    (judged, out)
}

#[cfg(test)]
mod tests;
