//! **How a frame is produced** — the offscreen input, the context run, and the
//! window that keeps its memory between frames.
//!
//! Split from [`super`] on the seam the two already have: nothing here
//! traverses a shape, and everything here hands a finished frame straight to
//! one of the parent's projections. So the walk stays exactly one
//! (`rules/no-hand-rolled-paint-walk.yml`) and this is the harness around it.

use super::text_of;

/// An offscreen input of exactly `w` × `h` logical points.
pub(crate) fn screen_sized(w: f32, h: f32) -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(w, h),
        )),
        ..Default::default()
    }
}

/// A screen big enough that every row lays out rather than scrolling away.
fn screen() -> egui::RawInput {
    screen_sized(1200.0, 2400.0)
}

/// Render `body` into a central panel on one throwaway frame and return
/// everything it painted, as text.
pub(crate) fn paint(mut body: impl FnMut(&mut egui::Ui)) -> String {
    let ctx = egui::Context::default();
    let output = ctx.run(screen(), |ctx| {
        egui::CentralPanel::default().show(ctx, &mut body);
    });
    text_of(&output)
}

/// **The window under test**: one persistent `egui::Context`, so focus, scroll
/// and hover carry frame to frame exactly as they do for the operator.
///
/// A pointer test needs this and cannot fake it: egui hit-tests a press against
/// the *previous* frame's widget rects, so a click delivered to a fresh context
/// tests against nothing at all.
pub(crate) struct Window {
    ctx: egui::Context,
    input: egui::RawInput,
}

impl Window {
    /// A window of the given size.
    pub(crate) fn sized(w: f32, h: f32) -> Self {
        Self {
            ctx: egui::Context::default(),
            input: screen_sized(w, h),
        }
    }

    /// A window big enough that nothing scrolls out.
    pub(crate) fn new() -> Self {
        Self::sized(1200.0, 2400.0)
    }

    /// Run one frame on `events` and hand back everything it produced.
    pub(crate) fn frame(
        &self,
        events: Vec<egui::Event>,
        mut body: impl FnMut(&egui::Context),
    ) -> egui::FullOutput {
        let mut input = self.input.clone();
        input.events = events;
        self.ctx.run(input, |ctx| body(ctx))
    }

    /// **Which widget holds the keyboard right now**, so a keyboard beat can
    /// say *which* control it walked onto rather than only that something has
    /// focus.
    pub(crate) fn focused(&self) -> Option<egui::Id> {
        self.ctx.memory(egui::Memory::focused)
    }

    /// One idle frame's text — what is on the glass right now.
    pub(crate) fn text(&self, body: impl FnMut(&egui::Context)) -> String {
        text_of(&self.frame(Vec::new(), body))
    }
}

/// **One full click at `pos`: move, press, release — three frames.** egui
/// hit-tests against the previous frame's widget rects, so a press in the frame
/// that first sees the pointer would test against nothing.
pub(crate) fn click(window: &Window, pos: egui::Pos2, mut body: impl FnMut(&egui::Context)) {
    window.frame(vec![egui::Event::PointerMoved(pos)], &mut body);
    for pressed in [true, false] {
        window.frame(vec![button(pos, pressed)], &mut body);
    }
}

/// The primary button going down, and coming back up.
fn button(pos: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    }
}

/// One key press, on no modifier plane.
pub(crate) fn press(key: egui::Key) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }
}

/// Run one idle frame on `window` and find the centre of the galley reading
/// exactly `label` — the coordinate a click is aimed at.
///
/// It runs the frame here rather than taking one, because a pointer test must
/// aim at the frame it is about to click into: egui hit-tests a press against
/// the previous frame's rects, and a coordinate read off some other frame is a
/// coordinate about some other window.
pub(crate) fn locate_in(
    window: &Window,
    label: &str,
    body: impl FnMut(&egui::Context),
) -> Option<egui::Pos2> {
    super::locate(&window.frame(Vec::new(), body), label)
}
