//! **A block's height, and the tail on its last line** (bl-2997): a single
//! line stands `ROW` tall and a block as tall as its lines with `XXS` of air
//! above and below — measurably shorter than a head centred in `ROW` over a
//! small line — and the tail stands at the last line's right end, in weak
//! ink at the small size, the line cut at the width it leaves.

use super::super::{Row, middle};
use crate::paint_probe::{Seen, fills_of, frame::Window, seen_of};
use crate::ui::theme::{INK, INK_WEAK, RAISED, ROW, glyph, space};

/// A chosen row with `beneath` under its head and `tail` at the end, laid
/// alone on a 300-wide window: the `RAISED` fill is the block's rect.
fn block(beneath: Option<&str>, tail: Option<&str>) -> egui::FullOutput {
    let window = Window::sized(300.0, 100.0);
    window.frame(Vec::new(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            Row {
                mark: ("?".to_owned(), INK),
                words: "head".to_owned(),
                ink: INK,
                state: None,
                chosen: true,
                indent: 0.0,
                aside: Some("2m".to_owned()),
                beneath: beneath
                    .map(|line| (line.to_owned(), INK_WEAK, egui::TextStyle::Small))
                    .into_iter()
                    .collect(),
                tail: tail.map(str::to_owned),
                door: None,
                help: "a block".to_owned(),
            }
            .paint(ui);
        });
    })
}

/// The one `RAISED` rect a frame filled.
fn raised(output: &egui::FullOutput) -> egui::Rect {
    fills_of(output)
        .into_iter()
        .find(|(_, fill)| *fill == RAISED)
        .expect("the chosen tint")
        .0
}

/// The run of `output` that begins with `text`.
fn run(output: &egui::FullOutput, text: &str) -> Seen {
    seen_of(output)
        .into_iter()
        .find(|run| run.text.starts_with(text))
        .unwrap_or_else(|| panic!("{text:?}"))
}

/// **A block is its lines plus `XXS` either side, and at least 15% shorter
/// than the head centred in `ROW` it replaced**; a single line keeps `ROW`,
/// and the head's words centre where [`middle`] says, in each shape.
#[test]
fn a_block_stands_as_tall_as_its_lines_and_a_line_keeps_row() {
    let window = Window::sized(300.0, 100.0);
    let mut heights = (0.0, 0.0, 0.0, 0.0);
    window.frame(Vec::new(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let own = |style: egui::TextStyle| {
                let font = style.resolve(ui.style());
                ui.fonts(|fonts| fonts.row_height(&font))
            };
            heights = (
                own(egui::TextStyle::Body),
                own(egui::TextStyle::Small),
                middle(ui, false),
                middle(ui, true),
            );
        });
    });
    let (body, small, single, blocked) = heights;
    assert!((single - ROW / 2.0).abs() < 0.01);
    assert!((blocked - (space::XXS + body / 2.0)).abs() < 0.01);
    let line = block(None, None);
    let one = raised(&line);
    assert!((one.height() - ROW).abs() < 0.01, "{}", one.height());
    assert!((run(&line, "? head").laid.center().y - (one.min.y + single)).abs() < 1.0);
    let output = block(Some("beneath"), Some("tail"));
    let two = raised(&output);
    let was = ROW + small + space::XXS;
    let now = 2.0 * space::XXS + body + small;
    assert!(
        (two.height() - now).abs() < 0.01,
        "{} ≠ {now}",
        two.height()
    );
    assert!(two.height() <= 0.85 * was, "{} against {was}", two.height());
    let head = run(&output, "? head").laid;
    assert!((head.center().y - (two.min.y + blocked)).abs() < 1.0);
}

/// **The tail stands at the last line's right end in weak ink**, on that
/// line, and the line is cut with `…` at the width the tail leaves it; with
/// nothing beneath there is no line to stand on and it is not painted.
#[test]
fn the_tail_ends_the_last_line_and_the_line_is_cut_before_it() {
    let long = "a preview whose first line runs on past the right edge of any bar";
    let output = block(Some(long), Some("tail"));
    let (block_rect, beneath, tail) = (
        raised(&output),
        run(&output, "a preview").laid,
        run(&output, "tail"),
    );
    assert_eq!(tail.ink, INK_WEAK);
    assert!(
        (tail.laid.max.x - (block_rect.max.x - space::S)).abs() < 1.0,
        "{tail:?}"
    );
    assert!(
        beneath.max.x <= tail.laid.min.x - space::S + 1.0,
        "{beneath:?}"
    );
    assert!((tail.laid.center().y - beneath.center().y).abs() < 1.0);
    assert!(
        run(&output, "a preview").text.ends_with(glyph::ELIDED),
        "cut at the width the tail leaves"
    );
    let bare = block(None, Some("tail"));
    assert!(
        !seen_of(&bare).iter().any(|run| run.text == "tail"),
        "nothing to stand on"
    );
}
