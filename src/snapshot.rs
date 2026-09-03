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
//! **Every size is judged, and there is no width-gate any more** (bl-dfda).
//! There was one — `promised`, a query into `crate::ui::shell::widths` asking
//! whether the layout still claimed a shape for a width — because the seat's
//! policy ran out of answers below the conversation's floor and asserting
//! geometry there would have asserted something nobody promised. The narrow
//! shape is that answer, `crate::ui::shell::policy::shape` returns one at every
//! width, and a gate whose condition is now *yes* is a gate that has stopped
//! reading. So it is gone rather than left saying yes, and the phone row is
//! judged by all four assertions with nothing here to switch it on.
//!
//! The images land under `target/`, which is untracked. That is deliberate on
//! two counts: an image is a derivation and re-derived by running the suite,
//! and this repository's disclosure gate refuses every tracked binary outright
//! (its allowlist is empty and stays empty).

use crate::ui::Model;
use egui_kittest::{Harness, HarnessBuilder};

pub(crate) mod blank;
pub(crate) mod clipped;
pub(crate) mod parity;
pub(crate) mod reach;
pub(crate) mod worlds;

mod tests;

/// **The viewport sizes the matrix renders at**, narrowest first.
///
/// `phone` is the size the ball named and the one that finds things: it is
/// under the width at which the broad layout can still leave the conversation
/// its floor, so it is the one size that renders in the narrow shape — one
/// column at a time (`crate::ui::shell::policy`). `narrow` is the width the
/// side panels have yielded to their limit at, and `desk` is the window with
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

/// The file one shot is written to, named so a directory listing IS the matrix.
pub(crate) fn name(world: &str, size: &str) -> String {
    format!("{world}--{size}.png")
}
