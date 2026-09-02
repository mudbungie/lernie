//! **The seat, rendered without a compositor** — and the invariants that read
//! the result rather than look at it (bl-dc07).
//!
//! # Why this exists beside [`crate::paint_probe`]
//!
//! The paint probe is the one walk over painted *glyphs*: it answers "what
//! words reached the glass, narrowed to what the clip rect let through". That
//! is the right altitude for almost everything this suite asserts, and it stays
//! the only walk of its kind — `rules/no-hand-rolled-paint-walk.yml` says so.
//!
//! It cannot answer two questions, and both are the gross-defect class an
//! operator sees in the first half second. **What does the window LOOK like**,
//! which needs pixels and so needs a rasterizer; and **what is reachable**,
//! which needs the accessibility tree, because a control an operator can act on
//! is not the same set as the runs of text that were laid out. The commit this
//! module was written on top of is the case in point: a seat painting a giant
//! black box in the middle of the window, which every glyph assertion passed
//! straight through.
//!
//! So this module adds a second harness rather than a second paint walk. It
//! runs the SAME `crate::ui::render` on an off-screen context — the one the
//! native boot runs, because `src/main.rs` decides nothing — and reads back the
//! two things the probe has nothing to say about.
//!
//! # PNGs are for eyes; invariants are for the gate
//!
//! The matrix in [`tests`] writes one PNG per (world, size) into [`shots`] and
//! **nothing compares them to anything**. A pinned golden image is a gate that
//! reddens on every font, theme and layout tweak, spends an afternoon per
//! rebaseline, and gets rebaselined without being looked at — which is a gate
//! that has stopped reading. What gates instead is three properties that hold
//! whatever the pixels are: [`reach`], [`blank`] and [`clipped`].
//!
//! The images land under `target/`, which is untracked. That is deliberate on
//! two counts: an image is a derivation and re-derived by running the suite,
//! and this repository's disclosure gate refuses every tracked binary outright
//! (its allowlist is empty and stays empty).

use crate::ui::Model;
use egui_kittest::{Harness, HarnessBuilder};

pub(crate) mod blank;
pub(crate) mod clipped;
pub(crate) mod reach;
pub(crate) mod worlds;

mod tests;

/// **The viewport sizes the matrix renders at**, narrowest first.
///
/// `phone` is the size the ball named and the one that finds things: at 400
/// points the two list panes are both on their floor
/// ([`crate::ui::shell::SIDE_FLOOR`]) and the chat pane is under its own, so
/// every pane is painting in less room than its layout asks for. `narrow` is
/// the width the side panels start yielding at, and `desk` is the window with
/// room to spare — the one where a defect hides.
pub(crate) const SIZES: [(&str, f32, f32); 3] = [
    ("phone", 400.0, 800.0),
    ("narrow", 900.0, 700.0),
    ("desk", 1400.0, 900.0),
];

/// **Where a rendered frame lands**, and the answer to "where do I look at it".
///
/// Off `CARGO_MANIFEST_DIR` rather than the working directory, the same way
/// [`crate::test_support::corpus`] resolves its own root: a test's working
/// directory is not a promise, and this path is documented in `README.md` for
/// an agent that has to find the file by name.
pub(crate) fn shots() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("snapshots")
}

/// **One settled frame of the whole window**, at one size, over one model.
///
/// The closure is `crate::ui::render` and nothing else, which is the point: the
/// thing under the camera is the window, not a rehearsal of it. `run` settles
/// the frame the way the native loop would — egui lays a panel out on the frame
/// after the one that measured it, so a single pass paints a window mid-layout.
pub(crate) fn seat(model: Model, width: f32, height: f32) -> Harness<'static, Model> {
    let mut harness = HarnessBuilder::<Model>::default()
        .with_size(egui::Vec2::new(width, height))
        .build_state(
            |ctx: &egui::Context, model: &mut Model| crate::ui::render(ctx, model),
            model,
        );
    harness.run();
    harness
}

/// **Whether the seat's own layout policy still promises this width a shape.**
///
/// Every size in [`SIZES`] is RENDERED, because a picture of a window that has
/// gone wrong is the most useful picture there is. Not every size is *judged*
/// on its geometry, and the line is not "what happens to pass" — it is
/// [`crate::ui::shell::widths`], which is where this seat states what it does
/// as a window narrows. The side panes yield in proportion until they reach
/// their own floor, and past that **nothing yields**: the chat pane goes under
/// [`crate::ui::shell::CHAT_FLOOR`] and the layout has said, in its own words,
/// that it has run out of answers.
///
/// So a width where the conversation still gets its floor is a width where a
/// control off the window is a defect. A width below it is a window the layout
/// never claimed to lay out, and asserting geometry there asserts something
/// nobody promised — while still, at 400 points, being photographed every run
/// for whoever fixes it. **What it is waiting for is bl-dfda**: the seat has no
/// narrow layout at all, and the phone row starts being judged, with no change
/// here, on the day that ball gives it one. **Assertion (a) is not gated by this**, and that is
/// the distinction rather than an exception: whether a control answers a click
/// is a question about the tree, and it holds at every width this seat opens
/// at.
///
/// It is a QUERY, not a second constant. A number written here would be a copy
/// of a policy that lives in one function, and the two would part company on
/// the first tuning of either.
pub(crate) fn promised(width: f32) -> bool {
    let (roster, convs) = crate::ui::shell::widths(width);
    width - roster - convs >= crate::ui::shell::CHAT_FLOOR
}

/// The file one shot is written to, named so a directory listing IS the matrix.
pub(crate) fn name(world: &str, size: &str) -> String {
    format!("{world}--{size}.png")
}
