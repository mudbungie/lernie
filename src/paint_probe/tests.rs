//! The walk itself: its recursion arm, its catch-all, and the one thing it
//! exists for — that it reads **painted glyphs** rather than the string that
//! went in.

use super::{collect_text, frame::Window, locate, seen_of, text_of};

/// **The whole reason this module exists.** A label too wide for its container
/// is elided by the toolkit, and `Galley::text()` would still report it whole —
/// so an assertion against the input string passes on a row that reached the
/// glass as `Po…`. The glyph read is what makes a dump evidence.
#[test]
fn a_run_the_toolkit_elided_reads_back_elided() {
    let window = Window::sized(90.0, 60.0);
    let painted = text_of(&window.frame(Vec::new(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(egui::Label::new("port the paint probe with the window").truncate());
        });
    }));
    assert!(!painted.contains("with the window"), "{painted:?}");
    assert!(
        painted.contains('…'),
        "the elision is what reached the glass: {painted:?}"
    );
}

/// The walk descends `Shape::Vec` and ignores everything that carries no text.
/// Built by hand, because simple widgets do not nest galleys in a vector — and
/// this is the ONE copy of a test that every pane would otherwise carry.
#[test]
fn the_walk_descends_a_shape_vec_and_ignores_what_carries_no_text() {
    let ctx = egui::Context::default();
    let mut nested: Option<egui::Shape> = None;
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        // Two rows, so the glyph walk's own line break is exercised beside its
        // single-row case: a galley's rows carry no newline glyph, so the walk
        // re-inserts one where the row says it ended on one.
        let galley = ctx.fonts(|f| {
            f.layout_no_wrap(
                "nested\nrows".into(),
                egui::FontId::default(),
                egui::Color32::WHITE,
            )
        });
        nested = Some(egui::Shape::Vec(vec![egui::Shape::Text(
            egui::epaint::TextShape {
                pos: egui::Pos2::ZERO,
                galley,
                underline: egui::Stroke::NONE,
                fallback_color: egui::Color32::WHITE,
                override_text_color: None,
                opacity_factor: 1.0,
                angle: 0.0,
            },
        )]));
    });
    let mut out = String::new();
    collect_text(nested.as_ref().expect("a shape"), &mut out);
    collect_text(&egui::Shape::Noop, &mut out);
    assert_eq!(out, "nested\nrows\n");
}

/// **The ink, both halves.** A coloured label names its colour in its layout
/// section; a plain one declines and egui resolves it at paint time from the
/// fallback — so a reader of the section alone is blind to half the frame.
#[test]
fn a_run_reports_the_colour_it_reached_the_glass_in() {
    let window = Window::new();
    let out = window.frame(Vec::new(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.colored_label(egui::Color32::RED, "named");
            ui.label("deferred");
        });
    });
    let seen = seen_of(&out);
    let inks: Vec<(String, egui::Color32)> = seen
        .iter()
        .map(|s| (s.text.clone(), s.ink))
        .filter(|(text, _)| text == "named" || text == "deferred")
        .collect();
    assert!(
        inks.contains(&("named".to_owned(), egui::Color32::RED)),
        "{inks:?}"
    );
    let deferred = inks
        .iter()
        .find(|(text, _)| text == "deferred")
        .expect("the plain run");
    assert_ne!(
        deferred.1,
        egui::Color32::PLACEHOLDER,
        "the fallback resolved"
    );
}

/// A galley clipped away entirely is not on the glass and is not evidence about
/// it, so it is dropped; one that fits is kept with the rect it was laid into.
#[test]
fn a_shape_clipped_away_entirely_is_not_seen() {
    let window = Window::sized(200.0, 40.0);
    let out = window.frame(Vec::new(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for n in 0..80 {
                    ui.label(format!("row {n}"));
                }
            });
        });
    });
    let seen: Vec<String> = seen_of(&out).into_iter().map(|s| s.text).collect();
    assert!(seen.iter().any(|t| t == "row 0"), "{seen:?}");
    assert!(!seen.iter().any(|t| t == "row 79"), "{seen:?}");
    assert!(text_of(&out).contains("row 0"));
}

/// **A run laid wider than its container is sliced, not elided** — mid-glyph,
/// with no ellipsis, because the galley was never truncated and so never had
/// one added. Only the two rects together tell that apart from a run that fits,
/// which is why both ride on a [`Seen`](super::Seen).
#[test]
fn a_run_wider_than_its_clip_is_shown_narrower_than_it_was_laid() {
    let window = Window::sized(120.0, 60.0);
    let out = window.frame(Vec::new(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(egui::Label::new("port the paint probe with the window").extend());
        });
    });
    let run = seen_of(&out)
        .into_iter()
        .find(|s| s.text.contains("port the paint probe"))
        .expect("the run");
    assert!(
        run.shown.width() < run.laid.width(),
        "laid {} but shown {}",
        run.laid.width(),
        run.shown.width()
    );
}

/// A click is aimed by painted glyphs. A label the frame never painted has no
/// coordinate, and saying so is what stops a test clicking at the origin.
#[test]
fn a_label_the_frame_never_painted_has_no_coordinate() {
    let window = Window::new();
    let out = window.frame(Vec::new(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("here");
        });
    });
    assert!(locate(&out, "here").is_some());
    assert!(locate(&out, "elsewhere").is_none());
}

/// **The fills projection, both directions** (bl-6952): a filled rectangle
/// reaches it with the colour it was filled in, and one clipped away entirely
/// does not — the rule [`seen_of`] holds for glyphs, held here for shapes.
///
/// It exists because a window puts more than text on a screen, and a defect
/// reported as *a large dark block* is not answerable from galleys at all. The
/// alternative was a second walk written wherever the question was asked, which
/// is what `rules/no-hand-rolled-paint-walk.yml` refuses.
#[test]
fn a_filled_rect_reaches_the_fills_projection_and_a_clipped_one_does_not() {
    let window = Window::sized(200.0, 100.0);
    let ink = egui::Color32::from_rgb(9, 9, 9);
    let out = window.frame(Vec::new(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();
            painter.rect_filled(
                egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(40.0, 20.0)),
                0.0,
                ink,
            );
            // Off the bottom of a 100-point window: laid, then clipped away.
            painter.rect_filled(
                egui::Rect::from_min_size(egui::pos2(10.0, 400.0), egui::vec2(40.0, 20.0)),
                0.0,
                ink,
            );
        });
    });
    let fills = super::fills_of(&out);
    let ours: Vec<_> = fills.iter().filter(|(_, fill)| *fill == ink).collect();
    assert_eq!(ours.len(), 1, "one of the two is on the glass: {ours:?}");
    assert!(
        (ours[0].0.width() - 40.0).abs() < 0.5,
        "the whole 40-point width is on the glass: {:?}",
        ours[0].0
    );
    assert!(
        fills.iter().any(|(_, fill)| *fill != ink),
        "the panel's own ground is a fill too, and the projection carries it"
    );
}
