//! **Assertion (c): nothing an operator can act on is outside the window.**
//!
//! A control laid out past the frame is the quietest gross defect there is. It
//! costs no error, reddens no glyph assertion — [`crate::paint_probe`] narrows
//! a run to its clip rect and simply reports less — and the window looks
//! finished. The operator's experience is that the thing they were told to
//! click is not anywhere.
//!
//! The subject is the **accessibility tree**, so the set being judged is the
//! set of things that are controls: a node the tree says can be clicked or
//! focused. A label that ran off the edge is a layout question and is not this
//! assertion's business.
//!
//! Two ways to fail, and the second is the one worth naming. A control whose
//! rectangle lies wholly outside the window cannot be reached with a pointer.
//! And a control the tree gives **no rectangle at all** cannot be aimed at
//! either — by a pointer, by a screen reader, or by the harness — which is the
//! same defect arriving through a different door.

use crate::ui::Model;
use egui_kittest::Harness;
use egui_kittest::kittest::{Queryable, by};

/// **The judgement, as a pure function of the geometry** — extracted from the
/// walk so both of its answers can be asked for directly. A detector whose
/// failing arm has never run is a detector nobody has evidence about.
///
/// The bounds arrive as four numbers rather than as a rectangle because the
/// type they come in belongs to a crate this one does not declare: the tree is
/// reached through the harness, and reaching past it for a type would be a
/// dependency taken by accident.
pub(crate) fn fault(
    bounds: Option<(f64, f64, f64, f64)>,
    width: f32,
    height: f32,
) -> Option<String> {
    let Some((x0, y0, x1, y1)) = bounds else {
        return Some("the tree gives it no rectangle, so nothing can aim at it".to_owned());
    };
    if x1 <= x0 || y1 <= y0 {
        return Some(format!("its rectangle is empty: {x0},{y0} to {x1},{y1}"));
    }
    if x1 <= 0.0 || y1 <= 0.0 || x0 >= f64::from(width) || y0 >= f64::from(height) {
        return Some(format!(
            "it lies wholly outside the {width}x{height} window: {x0},{y0} to {x1},{y1}"
        ));
    }
    None
}

/// **Every control the tree offers, judged.** An empty answer is the assertion
/// holding.
///
/// Hidden nodes are skipped, and that is not a loosening: `is_hidden` is the
/// tree's own statement that the node is not currently offered to anybody, so
/// judging it would be judging a control that is not on the window.
/// **Whether a control is scrolled past a box's clip** rather than laid off
/// the glass (DESIGN §4.42, item 6) — the overlay's box, or a rail's (§4.48
/// item 8): it stands within the box's columns and wholly above or below
/// the box's rows. Such a control is reachable by scrolling the pane it is
/// in — a Tab onto it brings it on — which is a different thing from a pane
/// that grew a control past the window with nothing to scroll.
pub(crate) fn scrolled(bounds: (f64, f64, f64, f64), clips: &[egui::Rect]) -> bool {
    let (x0, y0, x1, y1) = bounds;
    clips.iter().any(|clip| {
        let within = x0 >= f64::from(clip.min.x) - 1.0 && x1 <= f64::from(clip.max.x) + 1.0;
        within && (y0 >= f64::from(clip.max.y) || y1 <= f64::from(clip.min.y))
    })
}

/// **The boxes a pane scrolls in**, as the frame wrote them down: the
/// overlay's, and each rail's that stands (`crate::ui::rail::clip`).
pub(crate) fn clips(harness: &Harness<'_, Model>) -> Vec<egui::Rect> {
    [
        crate::ui::overlay::BOX,
        crate::ui::rail::clip(crate::ui::Rail::Branch),
        crate::ui::rail::clip(crate::ui::Rail::Context),
    ]
    .into_iter()
    .filter_map(|id| {
        harness
            .ctx
            .data(|d| d.get_temp::<egui::Rect>(egui::Id::new(id)))
    })
    .collect()
}

pub(crate) fn complaints(
    at: &str,
    width: f32,
    height: f32,
    harness: &Harness<'_, Model>,
) -> Vec<String> {
    let clips = clips(harness);
    harness
        .query_all(by())
        .filter(|node| !node.is_hidden() && (node.is_clickable() || node.is_focusable()))
        // **A label is not a control**, whatever the tree says about it: egui
        // marks selectable text as clickable so a pointer can select it, and
        // that made every sentence of a covering pane a thing this assertion
        // judged — so a pane could not grow a word of prose past the window
        // even inside its own scroll (bl-d1ae). The doc above already draws
        // the line: a label that ran off the edge is a layout question. The
        // role is read by its name because the type it comes in belongs to a
        // crate this one does not declare (`fault`'s own reasoning, above).
        .filter(|node| format!("{:?}", node.role()) != "Label")
        .filter_map(|node| {
            let bounds = node
                .bounding_box()
                .map(|rect| (rect.x0, rect.y0, rect.x1, rect.y1));
            if bounds.is_some_and(|bounds| scrolled(bounds, &clips)) {
                return None;
            }
            fault(bounds, width, height).map(|why| {
                let label = node.label().unwrap_or_else(|| "(no label)".to_owned());
                let role = node.role();
                format!("{at}: the {role:?} reading {label:?} — {why}")
            })
        })
        .collect()
}
