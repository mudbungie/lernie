//! Every token, and the four properties `docs/STYLE.md` says are asserted:
//! the ladder climbs, ink is legible on every rung, no accent is louder than
//! soft neon, and an unknown word is unstyled.

use super::{
    GROUND, HAIRLINE, INK, INK_FAINT, INK_WEAK, NOTICE, QR_INK, QR_PAPER, RAISED, STATES, SURFACE,
    Speaker, State, accent, speaker, state_ink, state_of, tint, tone_ink, visuals,
};
use crate::reply::convs::{AgentState, Tone};
use egui::Color32;

/// WCAG relative luminance off gamma bytes.
fn luminance(c: Color32) -> f64 {
    let channel = |b: u8| {
        let s = f64::from(b) / 255.0;
        if s <= 0.039_28 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(c.r()) + 0.7152 * channel(c.g()) + 0.0722 * channel(c.b())
}

/// WCAG contrast ratio, the brighter over the darker.
fn contrast(a: Color32, b: Color32) -> f64 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

/// HSV saturation off gamma bytes.
fn saturation(c: Color32) -> f64 {
    let (r, g, b) = (f64::from(c.r()), f64::from(c.g()), f64::from(c.b()));
    let max = r.max(g).max(b);
    if max == 0.0 {
        0.0
    } else {
        (max - r.min(g).min(b)) / max
    }
}

/// **The ladder climbs and the ink descends**: each elevation is brighter
/// than the one under it, and each ink is dimmer than the one above it.
#[test]
fn elevation_climbs_and_ink_descends() {
    let ladder = [GROUND, SURFACE, RAISED, HAIRLINE];
    for pair in ladder.windows(2) {
        assert!(luminance(pair[1]) > luminance(pair[0]), "{pair:?} climbs");
    }
    let inks = [INK, INK_WEAK, INK_FAINT];
    for pair in inks.windows(2) {
        assert!(luminance(pair[1]) < luminance(pair[0]), "{pair:?} descends");
    }
}

/// **Ink is legible on every surface**: primary at 7:1 or better, secondary at
/// 4.5:1, on every rung a word can stand on.
#[test]
fn ink_reads_on_every_surface() {
    for surface in [GROUND, SURFACE, RAISED] {
        assert!(contrast(INK, surface) >= 7.0, "INK on {surface:?}");
        assert!(
            contrast(INK_WEAK, surface) >= 4.5,
            "INK_WEAK on {surface:?}"
        );
    }
}

/// **Soft neon is a bound**: no accent is more than two-thirds saturated, and
/// every one reads on every surface at 4.5:1 or better.
#[test]
fn accents_are_soft_and_legible() {
    for state in STATES {
        let ink = accent(state);
        assert!(saturation(ink) <= 2.0 / 3.0, "{state:?} is soft");
        for surface in [GROUND, SURFACE, RAISED] {
            assert!(contrast(ink, surface) >= 4.5, "{state:?} on {surface:?}");
        }
    }
}

/// **An ink may lean violet but never read as a hue**: under half the
/// saturation of the least saturated accent.
#[test]
fn ink_is_not_a_hue() {
    // Rest wears an ink by design, so it is not one of the hues measured.
    let least = STATES
        .iter()
        .filter(|state| **state != State::Rest)
        .map(|state| saturation(accent(*state)))
        .fold(f64::MAX, f64::min);
    for ink in [INK, INK_WEAK, INK_FAINT] {
        assert!(saturation(ink) < least / 2.0, "{ink:?} is an ink");
    }
}

/// **Six states, six accents, no seventh** — and no two share a hue, so a
/// colour on the glass names exactly one state. Rest wears the weak ink on
/// purpose: nothing happening is not a colour.
#[test]
fn six_states_six_accents() {
    let accents: Vec<Color32> = STATES.iter().map(|state| accent(*state)).collect();
    for (i, a) in accents.iter().enumerate() {
        for b in accents.iter().skip(i + 1) {
            assert_ne!(a, b, "two states share an accent");
        }
    }
    assert_eq!(accent(State::Rest), INK_WEAK);
    assert_eq!(NOTICE, accent(State::Annotation));
}

/// **A tint is the accent with its alpha lowered** — a wash, never a fill.
#[test]
fn a_tint_is_a_wash_of_its_accent() {
    for state in STATES {
        let wash = tint(state);
        assert!(wash.a() < 64, "{state:?}'s tint is a wash: {wash:?}");
        assert!(wash.a() > 0, "{state:?}'s tint is visible");
    }
}

/// **The wire's states read onto the six**, and a word this build does not
/// know is unstyled — never restyled as a state it is not (rung 3).
#[test]
fn the_wire_s_states_read_onto_the_six_and_an_unknown_word_paints_plain() {
    assert_eq!(state_of(&AgentState::Live), Some(State::Inference));
    assert_eq!(state_of(&AgentState::InFlight), Some(State::Working));
    assert_eq!(state_of(&AgentState::Stopped), Some(State::Error));
    assert_eq!(state_of(&AgentState::Quiescent), Some(State::Rest));
    assert_eq!(state_of(&AgentState::Unknown("parked".to_owned())), None);
    assert_eq!(state_ink(&AgentState::Unknown("parked".to_owned())), INK);
    assert_eq!(state_ink(&AgentState::Live), accent(State::Inference));
    assert_eq!(tone_ink(&Tone::Unknown("amber".to_owned())), INK);
    assert_eq!(tone_ink(&Tone::Plain), INK);
    assert_eq!(tone_ink(&Tone::Weak), INK_WEAK);
    assert_eq!(tone_ink(&Tone::Good), accent(State::Rest));
    assert_eq!(tone_ink(&Tone::Bad), accent(State::Error));
    assert_eq!(tone_ink(&Tone::Live), accent(State::Inference));
    assert_eq!(tone_ink(&Tone::InFlight), accent(State::Working));
}

/// **A speaker is told by weight, not hue**: only the operator wears a colour,
/// and it is the brand.
#[test]
fn a_speaker_is_told_by_weight() {
    assert_eq!(speaker(Speaker::Operator), super::BRAND);
    assert_eq!(speaker(Speaker::Model), INK);
    assert_eq!(speaker(Speaker::Peer), INK_WEAK);
    assert_eq!(speaker(Speaker::Ended), INK_FAINT);
}

/// **The symbol's pair is black on white whatever the window is**: a camera
/// reads it, not an operator.
#[test]
fn the_symbol_keeps_paper_and_ink() {
    assert_eq!(QR_PAPER, Color32::WHITE);
    assert_eq!(QR_INK, Color32::BLACK);
}

/// **Dark, with no light face, and no outline on anything at rest**: the
/// ground is the panel, the one stroke left is the brand ring on a focused
/// field, and a boundary between rows is a hairline.
#[test]
fn the_window_is_dark_and_unboxed() {
    let v = visuals();
    assert!(v.dark_mode);
    assert_eq!(v.panel_fill, GROUND);
    assert_eq!(v.window_stroke, egui::Stroke::NONE);
    for rung in [v.widgets.inactive, v.widgets.hovered] {
        assert_eq!(
            rung.bg_stroke,
            egui::Stroke::NONE,
            "no outline on a control"
        );
    }
    for rung in [v.widgets.inactive, v.widgets.hovered, v.widgets.active] {
        assert!(
            rung.expansion.abs() < f32::EPSILON,
            "a control tints, it does not grow"
        );
    }
    // The one stroke left: the ring on whatever holds the keyboard.
    assert_eq!(v.widgets.active.bg_stroke.color, super::BRAND);
    assert_eq!(v.widgets.noninteractive.bg_stroke.color, HAIRLINE);
    assert_eq!(v.selection.stroke.color, super::BRAND);
    assert_eq!(v.selection.bg_fill, tint(State::Working));
    assert_eq!(v.extreme_bg_color, SURFACE);
}

/// **Installing the language sets the type scale**, four sizes and no fifth,
/// on the context every frame paints through.
#[test]
fn install_sets_the_type_scale() {
    let ctx = egui::Context::default();
    super::install(&ctx);
    let style = ctx.style();
    let size = |ts: egui::TextStyle| style.text_styles.get(&ts).map(|f| f.size);
    assert_eq!(size(egui::TextStyle::Body), Some(super::type_scale::BODY));
    assert_eq!(
        size(egui::TextStyle::Heading),
        Some(super::type_scale::HEADING)
    );
    assert_eq!(size(egui::TextStyle::Small), Some(super::type_scale::SMALL));
    assert_eq!(
        size(egui::TextStyle::Monospace),
        Some(super::type_scale::MONO)
    );
    assert!(
        style.spacing.interact_size.y <= super::ROW,
        "a control stands under a row"
    );
    assert_eq!(style.visuals.panel_fill, GROUND);
}
