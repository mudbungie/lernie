//! **Who is speaking, read off the glass** (STYLE §5): the rule beside each
//! block, the ink of the header over it, and the one row that wears a state
//! instead of a weight.
//!
//! A weight is not a word, so nothing in the row projection can be asked what
//! reached the operator — the rule is a fill and the header is an ink, and
//! both are only on the frame. These beats read the frame.

use crate::paint_probe::frame::Window;
use crate::paint_probe::seen_of;
use crate::reply::transcript::{Block, Entry, EntryKind, Transcript, Usage};
use crate::test_support::window::{numbered, said, seated};
use crate::ui::Model;
use crate::ui::theme;

/// One entry of the shape the wire hands over.
fn entry(kind: EntryKind) -> Entry {
    Entry {
        name: "003-entry.json".to_owned(),
        raw: "{}".to_owned(),
        kind,
    }
}

/// A model turn of one text block.
fn turn(model_id: &str, text: &str) -> Entry {
    entry(EntryKind::Model {
        model_id: model_id.to_owned(),
        blocks: vec![Block::Text(text.to_owned())],
        usage: Usage::new(),
    })
}

/// One tool result, its way and its body.
fn tool(content: &str, is_error: bool) -> Entry {
    entry(EntryKind::ToolResult {
        tool_use_id: "tu-1".to_owned(),
        content: content.to_owned(),
        is_error,
    })
}

/// The turn still open — a trail stands open while it runs (§4.45 item 5),
/// which is where a step's weight can be read.
fn running() -> Entry {
    entry(EntryKind::Streaming {
        thinking: String::new(),
        text: String::new(),
    })
}

/// One settled frame of the whole window over `entries`.
fn glass(entries: Vec<Entry>) -> egui::FullOutput {
    let mut model = Model {
        transcript: Transcript { entries },
        ..seated()
    };
    let window = Window::sized(1400.0, 900.0);
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model))
}

/// The run reading exactly `header` that stands beside a gutter glyph — the
/// block's own header, as against the same words on the rule under the
/// transcript or in the card over it (`crate::ui::status`, `chat::panel`).
fn run(output: &egui::FullOutput, header: &str) -> crate::paint_probe::Seen {
    let runs = seen_of(output);
    let gutters = [
        crate::ui::theme::glyph::PROMPT,
        crate::ui::theme::glyph::MODEL,
        crate::ui::theme::glyph::NOTE,
        crate::ui::theme::glyph::CALL,
        crate::ui::theme::glyph::DONE,
        crate::ui::theme::glyph::FAILED,
    ];
    runs.iter()
        .filter(|seen| seen.text == header)
        .find(|seen| {
            runs.iter().any(|glyph| {
                gutters.contains(&glyph.text.as_str())
                    && glyph.laid.max.x <= seen.laid.min.x
                    && glyph.laid.min.y <= seen.laid.center().y
                    && seen.laid.center().y <= glyph.laid.max.y
            })
        })
        .cloned()
        .unwrap_or_else(|| panic!("nothing on the glass reads {header:?} beside a gutter"))
}

/// The topmost run reading exactly `text` — a body, which stands under its
/// header rather than beside a gutter.
fn body(output: &egui::FullOutput, text: &str) -> crate::paint_probe::Seen {
    seen_of(output)
        .into_iter()
        .filter(|seen| seen.text == text)
        .min_by(|a, b| a.laid.min.y.total_cmp(&b.laid.min.y))
        .unwrap_or_else(|| panic!("nothing on the glass reads {text:?}"))
}

/// **The ink the words naming a speaker are painted in** — the weight,
/// which the gutter's glyph stands beside (DESIGN §4.42, item 4; there is
/// no rule beside a block any more).
fn worn(output: &egui::FullOutput, header: &str) -> egui::Color32 {
    run(output, header).ink
}

/// **The four weights, on the glass, in one transcript.** The operator's own
/// words wear the brand because the act is theirs, the model's wear full ink,
/// a peer's are weak and a sender the server buried is fainter still — and
/// none of the four is a hue, which is what leaves colour meaning state.
#[test]
fn each_speaker_s_block_wears_its_own_weight() {
    let ended = entry(EntryKind::Delivered {
        sender: "child".to_owned(),
        sender_name: None,
        epitaph: Some("delivered".to_owned()),
        body: "landed".to_owned(),
    });
    let output = glass(vec![
        said("op", "port it"),
        said("user", "and again"),
        said("judge-one", "seconded"),
        ended,
        turn("model-a", "the seam is real"),
    ]);
    for (header, weight) in [
        ("op", theme::speaker(theme::Speaker::Operator)),
        ("user", theme::speaker(theme::Speaker::Operator)),
        ("judge-one", theme::speaker(theme::Speaker::Peer)),
        ("child (delivered)", theme::speaker(theme::Speaker::Ended)),
        ("model-a", theme::speaker(theme::Speaker::Model)),
    ] {
        assert_eq!(
            worn(&output, header),
            weight,
            "{header:?} does not wear {weight:?}"
        );
    }
    // And the operator's is the brand itself, which is what "their own act" is
    // worn in everywhere else on this glass.
    assert_eq!(worn(&output, "op"), theme::BRAND);
    // The model's is body ink and not an accent: a speaker is never a hue.
    assert_eq!(worn(&output, "model-a"), theme::INK);
}

/// **A call that failed is the one row painted in a STATE**, because *this
/// will not mend itself* is a state and the five other rows on the glass are
/// speakers. A failure the operator has to read the body to spot is one they
/// will miss.
#[test]
fn a_failed_tool_result_wears_the_error_accent_and_a_returned_one_does_not() {
    let red = theme::accent(theme::State::Error);
    let output = glass(vec![tool("no such file", true), running()]);
    assert_eq!(worn(&output, "failed"), red);
    assert_eq!(body(&output, crate::ui::theme::glyph::FAILED).ink, red);
    let returned = glass(vec![tool("done", false), running()]);
    assert_ne!(worn(&returned, "returned"), red);
    assert_ne!(body(&returned, crate::ui::theme::glyph::DONE).ink, red);
}

/// **A folded head is dimmed**, so *there is more of this* is read off the
/// weight of the text before anybody reads the control that says so.
#[test]
fn a_folded_head_stands_in_weak_ink() {
    let output = glass(vec![tool(&numbered(700), false), running()]);
    let head = seen_of(&output)
        .into_iter()
        .find(|seen| seen.text.starts_with("line 001"))
        .expect("the head of the answer is on the glass");
    assert_eq!(head.ink, theme::INK_WEAK, "{:?}", head.text);
}

/// **Two of the model's turns are parted by air, not by a line** (bl-f251;
/// §4.42 keeps it — the hairline parts the OPERATOR's turns): the second
/// turn's gutter stands at least `S` under the first's words, and no
/// hairline-wide fill spans the column between them.
#[test]
fn entries_are_parted_by_air_and_never_by_a_line() {
    let output = glass(vec![turn("model-a", "first"), turn("model-a", "second")]);
    let first = body(&output, "first");
    let second = body(&output, "second");
    let gap = second.laid.min.y - first.laid.max.y;
    assert!(gap >= theme::space::S - 0.5, "gap {gap}");
    let dividers = crate::paint_probe::fills_of(&output)
        .into_iter()
        .filter(|(rect, _)| {
            rect.height() <= 1.5
                && rect.width() > 100.0
                && rect.min.y > first.laid.max.y
                && rect.max.y < second.laid.min.y
        })
        .count();
    assert_eq!(dividers, 0, "no line between two entries");
    // **And the body is prose**: laid at the leading, so one line of it
    // stands taller than the header over it.
    assert!(
        first.laid.height() >= theme::type_scale::LEADING - 0.5,
        "{:?}",
        first.laid
    );
}

/// **A wound ends the transcript in the error accent and names its mend**
/// (bl-d34f). It used to fall to the unknown-entry fallback and read
/// *«wound» (wounded, which this seat cannot read)* — a provider refusal
/// painted as a seat bug, with the server's own banner as its body.
#[test]
fn a_wound_wears_the_error_accent_and_names_the_verb_that_mends_it() {
    let output = glass(vec![entry(EntryKind::Wounded {
        wound: "refused".to_owned(),
        reason: None,
        auth_row: Some("anthropic".to_owned()),
    })]);
    let red = theme::accent(theme::State::Error);
    assert_eq!(worn(&output, crate::ui::chat::rows::WOUND), red);
    let said = crate::reply::transcript::wounded("refused", None, Some("anthropic"));
    assert!(
        said.contains("/login") && said.contains("anthropic"),
        "{said}"
    );
    assert_eq!(body(&output, &said).ink, theme::INK);
    assert!(
        seen_of(&output)
            .iter()
            .all(|seen| !seen.text.contains("cannot read")),
        "the fallback's sentence is not on the glass"
    );
}
