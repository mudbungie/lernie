//! **The two edges an operator can drag**, and the one line of egui that makes
//! either of them hold (DESIGN §4.39, bl-46e5).
//!
//! # Why the drag never worked, and it was not the design
//!
//! egui 0.30 stores a side panel's state as the rect its CONTENT took and
//! reads that back as the width on the next pass, and it floors that content
//! at the width RANGE's minimum and nothing else:
//!
//! ```text
//! ui.set_min_width((width_range.min - frame.inner_margin.sum().x).at_least(0.0));
//! ```
//!
//! (`containers/panel.rs`, `SidePanel::show_inside_dyn`). So a pane pulled
//! wider than its rows reported the rows' width and shrank to it on the next
//! frame. A pane whose body's FIRST act is `ui.set_min_width(ui.available_
//! width())` fills what it was given, the stored rect is the dragged one, and
//! the drag holds. `TopBottomPanel` does exactly that itself, one function
//! down in the same file, which is why the composer never had this half of the
//! defect.
//!
//! # What is the seat's, and what is still the policy's
//!
//! The panel's stored width is egui's memory and not this seat's fact. So a
//! width is taken as the operator's **only while they are dragging it**: on
//! every other frame the pane is handed the shown width EXACTLY
//! (`exact_width`, which is [`super::policy::shown`]'s answer clamped to the
//! window), and a width the policy imposed never comes back as a width the
//! operator set. That is the whole of *a narrower window clamps and never
//! overwrites*.

use crate::ui::shell::policy;

/// **Where the pointer has `id`'s resize handle right now**, or nothing.
///
/// Read off the PREVIOUS frame's response, which is exactly where the panel
/// reads it (egui 0.30 `containers/panel.rs` takes the resize interaction from
/// `read_response` before laying anything out), so this answer and the panel's
/// are one answer rather than two that can disagree by a frame.
fn held(ctx: &egui::Context, id: egui::Id) -> Option<egui::Pos2> {
    let resize = ctx.read_response(id.with("__resize"))?;
    if resize.dragged() {
        resize.interact_pointer_pos()
    } else {
        None
    }
}

/// **A list pane on an edge that drags.** It is shown at `want`; it hands back
/// the width the operator dragged it to, and nothing while nobody is dragging.
pub(super) fn side(
    ctx: &egui::Context,
    name: &'static str,
    want: f32,
    span: (f32, f32),
    body: impl FnOnce(&mut egui::Ui),
) -> Option<f32> {
    let holding = held(ctx, egui::Id::new(name)).is_some();
    let panel = egui::SidePanel::left(name).resizable(true);
    // **While a drag is in hand the range is the policy's floors**, so the
    // pointer moves the edge between them; on every other frame the width is
    // the shown one exactly, so a policy that has changed its mind outvotes
    // whatever the panel happened to store (bl-fef8's half of this).
    let panel = if holding {
        panel.width_range(egui::Rangef::new(span.0, span.1))
    } else {
        panel.exact_width(want)
    };
    let shown = panel.show(ctx, |ui| {
        ui.set_min_width(ui.available_width());
        body(ui);
    });
    holding.then(|| shown.response.rect.width())
}

/// **The composer's top edge**, and the rows it sets.
///
/// `body` paints the panel and hands back one line of body type, which is the
/// unit the answer is in: the drag is answered in ROWS and never in points, so
/// the panel goes on being handed a height rather than reading one back off
/// its content (DESIGN §4.38). `rows` is what the field stood at this frame,
/// which is what makes the rest of the panel's height measurable.
pub(super) fn bottom(
    ctx: &egui::Context,
    name: &'static str,
    rows: u8,
    body: impl FnOnce(&mut egui::Ui) -> f32,
) -> Option<u8> {
    let pointer = held(ctx, egui::Id::new(name));
    let shown = egui::TopBottomPanel::bottom(name)
        .resizable(true)
        .show(ctx, body);
    let line = shown.inner;
    let window = ctx.screen_rect();
    let chrome = shown.response.rect.height() - f32::from(rows) * line;
    let asked = window.bottom() - pointer?.y;
    Some(policy::rows(asked, chrome, line, window.height()))
}
