//! Inline spans and heading lines: the split, the faces, and the glass.

use super::{Span, job, spans};
use crate::paint_probe::frame::Window;
use crate::paint_probe::seen_of;
use crate::test_support::window::{said, seated};
use crate::ui::theme::type_scale;

fn words(s: &str) -> Span {
    Span::Words(s.to_owned())
}
fn code(s: &str) -> Span {
    Span::Code(s.to_owned())
}

/// **The split**: a span loses its ticks, an unpaired or empty one stays as
/// typed, a heading loses its marks only with the space after them and at
/// one to six of them, and the line breaks stay in the words.
#[test]
fn the_spans_and_the_headings_split_from_the_words() {
    assert_eq!(spans("plain"), vec![words("plain")]);
    assert_eq!(
        spans("run `ls -la` then `cd`"),
        vec![words("run "), code("ls -la"), words(" then "), code("cd")]
    );
    assert_eq!(spans("a ` lone tick"), vec![words("a ` lone tick")]);
    assert_eq!(spans("empty `` pair"), vec![words("empty `` pair")]);
    assert_eq!(
        spans("## The plan\nfirst `a`\n#not a heading\n####### seven"),
        vec![
            Span::Heading("The plan".to_owned()),
            words("\nfirst "),
            code("a"),
            words("\n#not a heading\n####### seven"),
        ]
    );
    assert_eq!(spans(""), Vec::<Span>::new());
}

/// **The faces**: words at the body, a span in the monospace family at MONO
/// on the surface, a heading at its size.
#[test]
fn each_run_wears_its_face() {
    let laid = job("# Head\nsay `x`", 300.0);
    let faces: Vec<(f32, egui::FontFamily)> = laid
        .sections
        .iter()
        .map(|s| (s.format.font_id.size, s.format.font_id.family.clone()))
        .collect();
    assert_eq!(
        faces,
        vec![
            (type_scale::HEADING, egui::FontFamily::Proportional),
            (type_scale::BODY, egui::FontFamily::Proportional),
            (type_scale::MONO, egui::FontFamily::Monospace),
        ]
    );
    assert_eq!(
        laid.sections[2].format.background,
        crate::ui::theme::SURFACE
    );
    assert!((laid.wrap.max_width - 300.0).abs() < f32::EPSILON);
}

/// **On the glass**: neither the ticks nor the hashes reach it.
#[test]
fn neither_ticks_nor_hashes_reach_the_glass() {
    let mut model = seated();
    model.transcript.entries = vec![said("user", "## Steps\nrun `make check` first")];
    let window = Window::new();
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    let output = window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    let runs = seen_of(&output);
    let prose = runs
        .iter()
        .find(|seen| seen.text.contains("make check"))
        .expect("the prose is on the glass");
    assert!(prose.text.starts_with("Steps"), "{:?}", prose.text);
    assert!(
        prose.text.contains("run make check first"),
        "{:?}",
        prose.text
    );
    assert!(runs.iter().all(|seen| !seen.text.contains('`')));
    assert!(runs.iter().all(|seen| !seen.text.contains("##")));
}
