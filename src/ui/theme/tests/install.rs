//! **The install** (`theme::install`): the type scale, four sizes and no
//! fifth, and what the toolkit holds of the settings (DESIGN §4.46).

use super::super::{GROUND, ROW, install, type_scale, wheel};
use crate::settings::{HELP_DELAY, Settings};

/// **Installing the language sets the type scale**, four sizes and no fifth,
/// on the context every frame paints through.
#[test]
fn install_sets_the_type_scale() {
    let ctx = egui::Context::default();
    install(&ctx, &Settings::default());
    let style = ctx.style();
    let size = |ts: egui::TextStyle| style.text_styles.get(&ts).map(|f| f.size);
    assert_eq!(size(egui::TextStyle::Body), Some(type_scale::BODY));
    assert_eq!(size(egui::TextStyle::Heading), Some(type_scale::HEADING));
    assert_eq!(size(egui::TextStyle::Small), Some(type_scale::SMALL));
    assert_eq!(size(egui::TextStyle::Monospace), Some(type_scale::MONO));
    assert!(
        style.spacing.interact_size.y <= ROW,
        "a control stands under a row"
    );
    assert_eq!(style.visuals.panel_fill, GROUND);
}

/// **Installing the language installs what the toolkit holds of the
/// settings** (DESIGN §4.46 items 1, 2 and 4): the wheel step, the delay
/// before a control tells — infinite while help is off — the scale, and
/// egui's own zoom chords off, so the knob is the one factor.
#[test]
fn install_installs_the_wheel_the_delay_and_the_scale() {
    let ctx = egui::Context::default();
    let settings = Settings {
        wheel: 180.0,
        help_delay: 1.5,
        scale: 1.5,
        ..Settings::default()
    };
    // Two frames: egui applies a new zoom factor at the head of the frame
    // after the one that asked for it.
    for _ in 0..2 {
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            install(ctx, &settings);
        });
    }
    assert!((ctx.options(|o| o.line_scroll_speed) - 180.0).abs() < f32::EPSILON);
    assert!(!ctx.options(|o| o.zoom_with_keyboard));
    assert!((ctx.style().interaction.tooltip_delay - 1.5).abs() < f32::EPSILON);
    assert!((ctx.zoom_factor() - 1.5).abs() < f32::EPSILON);
    let off = Settings {
        on: std::collections::BTreeSet::new(),
        ..Settings::default()
    };
    install(&ctx, &off);
    assert!(ctx.style().interaction.tooltip_delay.is_infinite());
    assert!(ctx.style().interaction.tooltip_grace_time.abs() < f32::EPSILON);
    install(&ctx, &Settings::default());
    assert!((ctx.style().interaction.tooltip_delay - HELP_DELAY).abs() < f32::EPSILON);
}

/// **The knob governs the swipe as well as the notch** (DESIGN §4.46 item 1;
/// bl-4519): a wheel event in points comes out of the hook's half multiplied
/// by `wheel / SWIPE`, a line-unit one is left to `line_scroll_speed`, and a
/// page-unit one is the screen's already.
#[test]
fn the_wheel_scales_a_swipe_and_leaves_a_notch_and_a_page_alone() {
    let settings = Settings {
        wheel: 250.0,
        ..Settings::default()
    };
    let event = |unit| egui::Event::MouseWheel {
        unit,
        delta: egui::vec2(2.0, -4.0),
        modifiers: egui::Modifiers::NONE,
    };
    let mut raw = egui::RawInput {
        events: vec![
            event(egui::MouseWheelUnit::Point),
            event(egui::MouseWheelUnit::Line),
            event(egui::MouseWheelUnit::Page),
            egui::Event::Zoom(1.5),
        ],
        ..egui::RawInput::default()
    };
    wheel(&mut raw, &settings);
    let deltas: Vec<egui::Vec2> = raw
        .events
        .iter()
        .filter_map(|e| match e {
            egui::Event::MouseWheel { delta, .. } => Some(*delta),
            _ => None,
        })
        .collect();
    assert_eq!(
        deltas,
        vec![
            egui::vec2(5.0, -10.0),
            egui::vec2(2.0, -4.0),
            egui::vec2(2.0, -4.0)
        ]
    );
    assert_eq!(raw.events.len(), 4, "nothing else is touched");
    // **At the default a swipe moves three times** — the ruling bl-4519 took.
    let mut raw = egui::RawInput {
        events: vec![event(egui::MouseWheelUnit::Point)],
        ..egui::RawInput::default()
    };
    wheel(&mut raw, &Settings::default());
    assert!(matches!(
        raw.events[0],
        egui::Event::MouseWheel { delta, .. } if delta == egui::vec2(6.0, -12.0)
    ));
}
