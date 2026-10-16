//! **A scroll the keyboard asked of the transcript** (DESIGN §4.46 item 3):
//! PageDown and PageUp a page, Ctrl+End and Ctrl+Home the ends — from the
//! box too, because the box has no cursor move for any of them.
//!
//! The ask is a view fact held where view facts are, the toolkit's own
//! memory (the dock's fold keeps its state the same way): the key handler
//! runs before any panel paints and the scroll area that owns the offset
//! paints later, so the request crosses the frame in the one store both
//! can reach, and is taken the frame it is applied. Nothing on the model
//! says where the transcript is scrolled to — the scroll area holds that —
//! and what it is scrolled to is written back out for one reader, the
//! suite ([`offset`]).

use egui::Id;

/// What was asked.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Scroll {
    /// A page on (`1.0`) or back (`-1.0`): the height the transcript shows.
    Page(f32),
    /// The end: the newest row.
    #[default]
    End,
    /// The head: the panel.
    Head,
}

/// Where the ask waits for the frame.
const ASKED: &str = "the transcript's scroll request";
/// Where the transcript writes how far down it stands, in points.
const OFFSET: &str = "the transcript's scroll offset";
/// **Farther than any transcript is tall**: an end is a delta the area
/// clamps, which is how egui's own scroll-to-end is spelled.
const WHOLE: f32 = 1.0e9;

/// **Ask for `scroll`** on the next frame the transcript paints.
pub fn request(ctx: &egui::Context, scroll: Scroll) {
    ctx.data_mut(|d| d.insert_temp(Id::new(ASKED), scroll));
}

/// **Whether an ask waits**, read by the scroll area as it is built: the
/// area sticks to its end every frame it is asked to, and egui's own wheel
/// unsticks it while a delta handed in from outside does not — so the frame
/// that spends an ask is built unstuck, and the area re-sticks itself the
/// moment the offset meets the end again, exactly as a wheel's does.
pub fn asked(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<Scroll>(Id::new(ASKED))).is_some()
}

/// **Spend the ask**, inside the transcript's scroll area: a page is the
/// height the area shows, an end is the whole, and the delta's sign is the
/// wheel's — a page on is the content moving up.
pub fn apply(ui: &egui::Ui) {
    let Some(scroll) = ui
        .ctx()
        .data_mut(|d| d.remove_temp::<Scroll>(Id::new(ASKED)))
    else {
        return;
    };
    let dy = match scroll {
        Scroll::Page(pages) => -pages * ui.clip_rect().height(),
        Scroll::End => -WHOLE,
        Scroll::Head => WHOLE,
    };
    ui.scroll_with_delta(egui::vec2(0.0, dy));
}

/// **Note how far down the transcript stands**, for [`offset`].
pub fn note(ctx: &egui::Context, offset: f32) {
    ctx.data_mut(|d| d.insert_temp(Id::new(OFFSET), offset));
}

/// **How far down the transcript stood on the last frame it painted**, in
/// points, or nothing where none has.
pub fn offset(ctx: &egui::Context) -> Option<f32> {
    ctx.data(|d| d.get_temp(Id::new(OFFSET)))
}
