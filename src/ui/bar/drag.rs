//! **The bar's right edge, dragged** (DESIGN §4.45 item 2; bl-7578): both
//! widths are the operator's where they dragged them and the policy's
//! defaults where they did not, and the policy keeps the floor and the
//! ceiling by clamping what is SHOWN on every frame and never by overwriting
//! what was set (§4.39: *a window narrower than the stored width clamps it
//! and does not overwrite it*).
//!
//! **Which width a drag writes is which state the bar was in when the drag
//! began** ([`Drag::open`]), held from the drag's first frame to its last —
//! so a bar that opens under the pointer mid-drag does not hand the drag to
//! the other width. The width it began at is held beside it, and every frame
//! of the drag asks for that width moved by the pointer's whole travel since
//! the press: a drag that crosses a bound and comes back finds the edge
//! waiting where the pointer re-crosses it, with nothing accumulated past it.
//!
//! The seat is the seat's own strip and not egui's resizable panel: the
//! panel's width is eased on egui's clock (§4.44 item 1), and a resizable
//! panel stores a width of its own that would fight the eased value.

use crate::ui::theme;

/// **How wide the drag seat at the bar's right edge is**, in points: a strip
/// the panel keeps clear, so nothing a click takes stands under it.
pub const HANDLE: f32 = theme::space::S;

/// **The two widths the operator dragged**, in points, each `None` until they
/// have — the defaults being [`super::SHUT`] and [`super::open_width`].
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Dragged {
    /// The collapsed width.
    pub shut: Option<f32>,
    /// The expanded width.
    pub open: Option<f32>,
}

impl Dragged {
    /// **The two widths as shown in a window `window` wide**, the glyph
    /// column's floor being `floor`: the expanded one between the floor and
    /// the [`ceiling`], the collapsed one between the floor and the expanded
    /// one. Clamped here, on every frame, and written nowhere.
    pub fn shown(self, window: f32, floor: f32) -> (f32, f32) {
        let open = self
            .open
            .unwrap_or_else(|| super::open_width(window))
            .min(ceiling(window))
            .max(floor);
        let shut = self.shut.unwrap_or(super::SHUT).min(open).max(floor);
        (shut, open)
    }
}

/// **A drag in hand**: which width it writes, and where that width stood
/// when it began.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Drag {
    /// The bar was open when the edge was grabbed, so the expanded width is
    /// the one written.
    pub open: bool,
    /// The width written, as it stood on the drag's first frame.
    pub from: f32,
}

/// **The most the expanded width may be** in a window `window` wide: two
/// fifths of it (§4.44 item 1).
pub fn ceiling(window: f32) -> f32 {
    window * 2.0 / 5.0
}

/// **The least the collapsed width may be**: one glyph cell and the panel's
/// margins. The cell is a square of the body's line height — no glyph of a
/// text font is wider than its line is tall — so the floor is read off the
/// font in use rather than guessed from one glyph.
pub fn floor(ctx: &egui::Context) -> f32 {
    let body = egui::TextStyle::Body.resolve(&ctx.style());
    let cell = ctx.fonts(|fonts| fonts.row_height(&body));
    cell + margin().sum().x
}

/// **The panel's inner margin**: the usual air on three sides, and the seat's
/// width besides on the right, so the rows end where the seat begins.
pub fn margin() -> egui::Margin {
    egui::Margin {
        left: theme::space::XS,
        right: theme::space::XS + HANDLE,
        top: theme::space::XS,
        bottom: theme::space::XS,
    }
}

/// **The seat at the bar's right edge**: a strip [`HANDLE`] wide standing the
/// panel's full height at `width`, under the resize cursor, lit in the hover
/// wash while the pointer is on it or it is in hand (§4.45 item 6). It writes
/// the width the drag began on, clamped to that width's own bounds: the
/// collapsed one to the floor and the expanded width, the expanded one to the
/// collapsed width and the ceiling.
pub(super) fn edge(ui: &mut egui::Ui, bar: &mut super::Bar, open: bool, width: f32) {
    let screen = ui.ctx().screen_rect();
    let strip = egui::Rect::from_min_max(
        egui::pos2(screen.left() + width - HANDLE, screen.top()),
        egui::pos2(screen.left() + width, screen.bottom()),
    );
    // **Not a Tab stop**: egui's `Sense::drag()` is focusable, and a seat
    // only a pointer can move would be one more stop on the keyboard's walk
    // through the bar (`keys::tests::focus`) that no key can do anything at.
    let sense = egui::Sense {
        click: false,
        drag: true,
        focusable: false,
    };
    let seat = ui
        .interact(strip, egui::Id::new("bar edge"), sense)
        .on_hover_cursor(egui::CursorIcon::ResizeHorizontal);
    if seat.hovered() || seat.dragged() {
        ui.painter().rect_filled(strip, 0.0, theme::hover());
    }
    let floor = floor(ui.ctx());
    let (shut, wide) = bar.dragged.shown(screen.width(), floor);
    if seat.drag_started() {
        let from = if open { wide } else { shut };
        bar.drag = Some(Drag { open, from });
    }
    let origin = ui.ctx().input(|i| i.pointer.press_origin());
    if let (Some(drag), Some(origin), Some(at)) = (bar.drag, origin, seat.interact_pointer_pos()) {
        let asked = drag.from + at.x - origin.x;
        if drag.open {
            bar.dragged.open = Some(asked.min(ceiling(screen.width())).max(shut));
        } else {
            bar.dragged.shut = Some(asked.min(wide).max(floor));
        }
    }
    if seat.drag_stopped() {
        bar.drag = None;
    }
}
