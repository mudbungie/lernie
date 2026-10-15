//! The bar: its two widths and the clock between them, the pin, its rows,
//! its head's `+ new`, and what it says with nothing to list.

use super::{Bar, DESK, EMPTY, LINGER, NOTHING_AIMED, open_width};
use crate::paint_probe::{Seen, frame::Window, seen_of};
use crate::test_support::window::{conv, seated};
use crate::ui::{Model, theme};

/// A row is its whole block.
mod block;
/// What bounds the two widths: the defaults, the floor, the ceiling.
mod bounds;
/// The edge, dragged: which width a drag writes, and the state it keeps.
mod drag;
/// The edge's ease between its two widths.
mod ease;
/// The workspace at the foot, and its list.
mod foot;
/// The head's two doors, at the heading size with air inside the seat.
mod head;

/// **One frame at `time` on the input clock**, the pointer at `at` (or gone),
/// on a desk-sized window — what the bar's clock is steered by.
pub(super) fn tick(
    ctx: &egui::Context,
    model: &mut Model,
    time: f64,
    at: Option<egui::Pos2>,
) -> egui::FullOutput {
    let moved = at.map_or(egui::Event::PointerGone, egui::Event::PointerMoved);
    step(ctx, model, DESK, time, vec![moved])
}

/// **One frame at `time` carrying `events`**, on a window `width` wide and
/// 900 tall — [`tick`] with the pointer's buttons and the window's width in
/// hand, which a drag needs.
pub(super) fn step(
    ctx: &egui::Context,
    model: &mut Model,
    width: f32,
    time: f64,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    let mut input = crate::paint_probe::frame::screen_sized(width, 900.0);
    input.time = Some(time);
    input.events = events;
    ctx.run(input, |ctx| crate::ui::render(ctx, model))
}

/// The runs the bar put on the glass: everything left of its open edge.
pub(super) fn bar(output: &egui::FullOutput) -> Vec<Seen> {
    seen_of(output)
        .into_iter()
        .filter(|run| run.shown.min.x < open_width(DESK))
        .collect()
}

/// The seated conversation's open row, glyph and name.
pub(super) const ROW: &str = "✔ port the paint probe";

/// **The width the bar's panel stood at** on the last frame.
pub(super) fn panel(ctx: &egui::Context) -> f32 {
    egui::containers::panel::PanelState::load(ctx, egui::Id::new("bar"))
        .expect("the bar stood")
        .rect
        .width()
}

/// **Click the bar's run reading `text`** — aimed off the glyphs the bar
/// painted, since the same words can stand on the column too.
pub(super) fn click_bar(window: &Window, model: &mut Model, text: &str) {
    let mut body = |ctx: &egui::Context| crate::ui::render(ctx, model);
    let at = bar(&window.frame(Vec::new(), &mut body))
        .into_iter()
        .find(|run| run.text == text)
        .unwrap_or_else(|| panic!("the bar says {text:?}"))
        .shown
        .center();
    crate::paint_probe::frame::click(window, at, &mut body);
}

/// The texts of [`bar`]'s runs.
pub(super) fn said(output: &egui::FullOutput) -> Vec<String> {
    bar(output).into_iter().map(|run| run.text).collect()
}

/// **Whether the bar stood open on `output`**: its head says the pin's word
/// open and the glyph alone shut, from the first frame — the signal that
/// waits on neither the eased width nor a font's measure of a name, now a
/// name is elided at whatever width the bar has (bl-773e).
pub(super) fn opened(output: &egui::FullOutput) -> bool {
    let pin = theme::worded(theme::glyph::PIN, super::PIN);
    let unpin = theme::worded(theme::glyph::PIN, super::UNPIN);
    said(output).iter().any(|t| *t == pin || *t == unpin)
}

/// **The clock, as a value**: the pointer over keeps it open; leaving starts
/// the linger, which asks for the repaint at the moment it lapses; at
/// [`LINGER`] — 0.75 s, the number read off the ruling (DESIGN §4.45 item
/// 1) — it shuts and forgets; and the pin holds it open with no repaint owed.
#[test]
fn the_linger_runs_from_when_the_pointer_left() {
    assert!((LINGER - 0.75).abs() < f64::EPSILON, "the ruling's number");
    let mut bar = Bar::default();
    assert!(!bar.open(0.0), "shut at rest");
    assert_eq!(bar.heard(0.0, false), None);
    assert_eq!(bar.heard(1.0, true), None);
    assert!(bar.open(1.0));
    assert_eq!(bar.heard(1.5, false), Some(0.25), "a quarter second left");
    assert!(bar.open(1.5), "lingering");
    assert!(!bar.open(1.0 + LINGER));
    assert_eq!(bar.heard(1.0 + LINGER, false), None);
    assert_eq!(bar.left_at, None, "forgotten once lapsed");
    bar.pinned = true;
    assert!(bar.open(9.0), "pinned");
    bar.left_at = Some(9.0);
    assert_eq!(bar.heard(9.5, false), None, "a pinned bar owes no repaint");
}

/// **The pointer expands it the frame it enters, and it lingers 0.75 s after
/// it leaves**, then shuts on the repaint it asked for.
#[test]
fn the_pointer_opens_it_and_it_lingers_after() {
    let ctx = egui::Context::default();
    let mut model = seated();
    let open = opened;
    assert!(!open(&tick(&ctx, &mut model, 0.0, None)), "shut at rest");
    let over = Some(egui::pos2(10.0, 300.0));
    tick(&ctx, &mut model, 1.0, over);
    let eased = 1.0 + f64::from(super::EASE);
    assert!(open(&tick(&ctx, &mut model, eased, over)), "open under it");
    let inside = Some(egui::pos2(300.0, 300.0));
    assert!(
        open(&tick(&ctx, &mut model, 3.0, inside)),
        "a pointer resting in the open bar keeps it open"
    );
    let away = Some(egui::pos2(900.0, 300.0));
    let left = tick(&ctx, &mut model, 3.25, away);
    assert!(open(&left), "lingering");
    let owed = left.viewport_output[&egui::ViewportId::ROOT].repaint_delay;
    assert!(
        owed <= std::time::Duration::from_secs_f64(LINGER),
        "{owed:?}"
    );
    // Last over at 3.0, so it shuts at 3.75.
    assert!(open(&tick(&ctx, &mut model, 3.7, away)), "still lingering");
    assert!(!open(&tick(&ctx, &mut model, 3.8, away)), "lapsed");
    assert!(
        !open(&tick(&ctx, &mut model, 6.0, None)),
        "the pointer gone"
    );
}

/// **Pinned, it stands open whatever the pointer does**; the pin is one
/// toggle wearing the act it would do, and the collapsed head shows its
/// glyph alone.
#[test]
fn the_pin_holds_it_open_and_says_what_it_would_do() {
    let window = Window::new();
    let mut model = seated();
    let mut body = |ctx: &egui::Context| crate::ui::render(ctx, &mut model);
    let shut = window.text(&mut body);
    assert!(shut.lines().any(|line| line == theme::glyph::PIN), "{shut}");
    crate::test_support::window::click(&window, theme::glyph::PIN, &mut body);
    assert!(model.bar.pinned);
    let ctx = egui::Context::default();
    let pinned = said(&tick(&ctx, &mut model, 0.0, None));
    let unpin = theme::worded(theme::glyph::PIN, super::UNPIN);
    assert!(pinned.contains(&unpin), "{pinned:?}");
    assert!(pinned.contains(&ROW.to_owned()), "{pinned:?}");
    model.bar.pinned = false;
    model.bar.left_at = Some(0.0);
    let open = said(&tick(&ctx, &mut model, 0.5, None));
    let pin = theme::worded(theme::glyph::PIN, super::PIN);
    assert!(open.contains(&pin), "{open:?}");
}

/// **An open row**: the glyph in its state's ink, the name in body ink (the
/// selected one's in the brand), the age at the right end and the preview
/// beneath in weak ink, indented by its depth; and a click selects it.
#[test]
fn an_open_row_says_its_state_name_age_and_first_line_and_a_click_selects() {
    let mut model = seated();
    model.bar.pinned = true;
    let mut child = conv("20260830T051300Z-c3d4", "the child");
    child.depth = 1;
    child.preview = "first line\nsecond line".to_owned();
    child.attention = 1;
    model.convs.push(child);
    model.opened.insert("20260830T051200Z-a1b2".to_owned());
    let ctx = egui::Context::default();
    let runs = bar(&tick(&ctx, &mut model, 0.0, None));
    let find = |text: &str| {
        runs.iter()
            .find(|run| run.text == text)
            .unwrap_or_else(|| panic!("{text:?} in {runs:?}"))
    };
    let selected = find(ROW);
    let quiet = theme::state_ink(&crate::reply::convs::AgentState::Quiescent);
    assert_eq!(selected.ink, quiet, "the glyph in its state's ink");
    let asking = find("? the child");
    assert_eq!(asking.ink, theme::accent(theme::State::Attention));
    assert!(asking.laid.min.x > selected.laid.min.x, "indented");
    assert_eq!(find("42s").ink, theme::INK_WEAK);
    assert_eq!(find("first line").ink, theme::INK_WEAK);
    assert!(!runs.iter().any(|run| run.text.contains("second line")));
    let window = Window::sized(DESK, 900.0);
    let mut body = |ctx: &egui::Context| crate::ui::render(ctx, &mut model);
    crate::test_support::window::click(&window, "? the child", &mut body);
    assert_eq!(model.conversation.as_deref(), Some("20260830T051300Z-c3d4"));
}

/// **What it says with nothing to list**, open: the wall answered empty, the
/// wall not answered yet, and nothing aimed — and collapsed, nothing.
#[test]
fn an_empty_list_says_why_open_and_nothing_shut() {
    let ctx = egui::Context::default();
    let mut model = seated();
    model.convs.clear();
    model.conversation = None;
    model.answered.clone_from(&model.aim);
    let shut = said(&tick(&ctx, &mut model, 0.0, None));
    assert!(!shut.iter().any(|t| t == EMPTY), "{shut:?}");
    model.bar.pinned = true;
    let empty = said(&tick(&ctx, &mut model, 0.0, None));
    assert!(empty.iter().any(|t| t == EMPTY), "{empty:?}");
    model.answered = None;
    let waiting = said(&tick(&ctx, &mut model, 0.0, None));
    assert!(
        waiting.iter().any(|t| t == crate::ui::convs::NOT_ANSWERED),
        "{waiting:?}"
    );
    let mut nowhere = Model::default();
    nowhere.bar.pinned = true;
    let unaimed = said(&tick(&ctx, &mut nowhere, 0.0, None));
    assert!(unaimed.iter().any(|t| t == NOTHING_AIMED), "{unaimed:?}");
    assert!(!unaimed.iter().any(|t| t == crate::ui::roster::wall::NEW));
}

/// **`+ new` at the head begins a conversation on the aimed wall** — the
/// switcher's own door — and stands down under a covering pane but the
/// switcher's; nothing of the bar paints under the enrollment.
#[test]
fn new_begins_on_the_aimed_wall_and_stands_down_under_a_cover() {
    let window = Window::new();
    let mut model = seated();
    model.switching = true;
    let ctx = egui::Context::default();
    let shut = said(&tick(&ctx, &mut model, 0.0, None));
    assert!(shut.iter().any(|t| t == super::PLUS), "collapsed: {shut:?}");
    // **The pointer opens the bar before it can click**, so the click lands
    // on the open head's words.
    model.bar.pinned = true;
    click_bar(&window, &mut model, crate::ui::roster::wall::NEW);
    assert_eq!(model.conversation, None, "the selection is put down");
    assert!(!model.switching, "the switcher closed on the way");
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(
        window.focused(),
        Some(egui::Id::new(crate::ui::keys::BOX_ID)),
        "the caret is in the box that begins one"
    );
    model.begin_trail();
    model.bar.pinned = false;
    let covered = said(&tick(&ctx, &mut model, 0.0, None));
    assert!(!covered.iter().any(|t| t == super::PLUS), "{covered:?}");
    let mut enrolling = seated();
    enrolling.enroll = enrolling.aim.clone().map(crate::ui::Enrolling::at);
    let none = said(&tick(&ctx, &mut enrolling, 0.0, None));
    assert!(!none.iter().any(|t| t == theme::glyph::PIN), "{none:?}");
}

/// **The rows wear the switcher's tags, `+ new` the start's, and the pin
/// none** (DESIGN §4.44: the pin posts nothing).
#[test]
fn the_rows_and_new_wear_their_tags_and_the_pin_none() {
    use egui_kittest::kittest::Queryable;
    let mut model = seated();
    model.bar.pinned = true;
    let harness = crate::snapshot::seat(model, DESK, 900.0);
    let tag = |label: &str| {
        harness
            .query_all_by_label(label)
            .find_map(|node| node.author_id().map(str::to_owned))
    };
    let prefix = crate::ui::act::PREFIX;
    let row = tag(ROW).expect("the row is tagged");
    assert!(row.starts_with(&format!("{prefix}{}", crate::verbs::TRANSCRIPT.word)));
    let new = tag(crate::ui::roster::wall::NEW).expect("+ new is tagged");
    assert!(new.contains(crate::verbs::PREPARE), "{new}");
    let unpin = theme::worded(theme::glyph::PIN, super::UNPIN);
    assert_eq!(tag(&unpin), None, "the pin posts nothing");
}
