//! Fenced code in what was said: the split, and the glass.

use super::{Part, parts};
use crate::paint_probe::frame::Window;
use crate::paint_probe::seen_of;
use crate::test_support::window::{said, seated};

/// **The split**: prose around a block, the info string and the fences
/// dropped, empty prose between blocks not a part, and a fence still open —
/// a streaming turn — running to the end.
#[test]
fn the_fences_split_prose_from_code() {
    assert_eq!(
        parts("just words"),
        vec![Part::Prose("just words".to_owned())]
    );
    assert_eq!(
        parts("run this:\n```sh\nls -la\necho hi\n```\n\n```\nb\n```\nand done"),
        vec![
            Part::Prose("run this:".to_owned()),
            Part::Code("ls -la\necho hi".to_owned()),
            Part::Code("b".to_owned()),
            Part::Prose("and done".to_owned()),
        ]
    );
    assert_eq!(
        parts("streaming\n```rust\nfn main() {"),
        vec![
            Part::Prose("streaming".to_owned()),
            Part::Code("fn main() {".to_owned()),
        ]
    );
    assert_eq!(parts(""), Vec::<Part>::new());
}

/// **On the glass**: the block's words reach it in the monospace family, the
/// fence does not reach it at all, the prose beside it stays proportional,
/// and a line far wider than the window is held inside it.
#[test]
fn a_block_reaches_the_glass_as_code() {
    let long = "x".repeat(400);
    let mut model = seated();
    model.transcript.entries = vec![said("user", &format!("look:\n```sh\nls -la\n{long}\n```"))];
    let window = Window::new();
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    let output = window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    let runs = seen_of(&output);
    let code = runs
        .iter()
        .find(|seen| seen.text.starts_with("ls -la"))
        .expect("the block is on the glass");
    assert_eq!(code.family, egui::FontFamily::Monospace);
    assert!(code.text.contains(&long), "a code line is not wrapped");
    assert!(
        code.shown.width() < code.laid.width(),
        "held inside the column"
    );
    let prose = runs
        .iter()
        .find(|seen| seen.text == "look:")
        .expect("the prose is on the glass");
    assert_eq!(prose.family, egui::FontFamily::Proportional);
    assert!(runs.iter().all(|seen| !seen.text.contains("```")));
}
