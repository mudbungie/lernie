//! **The box's trailing end** (DESIGN §4.48 item 4): the conversation's
//! model and effort selectors as pills, then the send door — the baseline's
//! composer, where the two knobs an operator reaches for mid-sentence stand
//! where the sentence is.
//!
//! The pills are the selectors the status bar used to carry, unchanged: the
//! same doors (`Selector::Model`, `Selector::Effort`), words, `⏶ ↻ ✔`
//! marks and dropups (`crate::ui::status::dropup`), which now open upward
//! from the box's top edge. The send door fires what Enter fires; the
//! caller decides what that is, so there is one deposit door and not two.

use crate::ui::status::{self, Door, Run};
use crate::ui::{Model, Selector, theme};

/// **The send door of one mode of the box**: whether there is anything to
/// send, what the door does, and the ops it spends.
pub struct Send {
    pub live: bool,
    pub help: &'static str,
    pub ops: &'static [&'static str],
}

/// **What the trailing end holds this frame**: the pills, the cut the
/// model's words are held to, and the send door where the mode has one.
pub struct Trail {
    pills: Vec<Run>,
    cap: f32,
    send: Option<Send>,
}

/// **What a pointer did at the trailing end**: the pill clicked, where each
/// pill stood (a dropup's anchor), and whether the door was pressed.
#[derive(Default)]
pub struct Clicks {
    picked: Option<Selector>,
    seats: Vec<(Selector, egui::Rect)>,
    pub sent: bool,
}

/// The air inside a pill, each side.
const AIR: egui::Vec2 = egui::vec2(theme::space::S, theme::space::XXS);

impl Trail {
    /// The trailing end of a box `ui` is about to lay: the pills where
    /// `pilled` and a workspace is aimed, and `send`. **Each pill's words
    /// are cut to a quarter of the box's inside.**
    pub fn new(ui: &egui::Ui, model: &Model, pilled: bool, send: Option<Send>) -> Self {
        let pills = if pilled {
            status::selectors::pills(model)
        } else {
            Vec::new()
        };
        Trail {
            pills,
            cap: (ui.available_width() - 2.0 * theme::space::S) / 4.0,
            send,
        }
    }

    /// One pill's words, cut to the cap.
    fn words(&self, ui: &egui::Ui, run: &Run) -> String {
        status::elide(ui, &run.text, self.cap)
    }

    /// One pill's width, its seat's air inside it.
    fn pill(&self, ui: &egui::Ui, run: &Run) -> f32 {
        let mark = run
            .mark
            .map_or(String::new(), |(glyph, _)| format!(" {glyph}"));
        status::width(ui, &format!("{}{mark}", self.words(ui, run))) + 2.0 * AIR.x
    }

    /// **The width the trailing end takes**, which the field leaves it.
    pub fn width(&self, ui: &egui::Ui) -> f32 {
        let gap = ui.spacing().item_spacing.x;
        let pills: f32 = self.pills.iter().map(|run| self.pill(ui, run) + gap).sum();
        pills + self.send.as_ref().map_or(0.0, |_| theme::ROW + gap)
    }

    /// **Lay the pills and the door**, in reading order, noting what the
    /// pointer did in `clicks`. Where one would run past the line's end it
    /// starts the next (`theme::paint::prompt` lays them on a wrapping line
    /// of their own on a box too narrow for them beside the field): a pill
    /// is a frame, which egui's own wrapping cannot move.
    pub fn paint(&self, ui: &mut egui::Ui, clicks: &mut Clicks) {
        let wrap = |ui: &mut egui::Ui, wide: f32| {
            if wide > ui.available_size_before_wrap().x {
                ui.end_row();
            }
        };
        for run in &self.pills {
            wrap(ui, self.pill(ui, run));
            let words = self.words(ui, run);
            let stood = egui::Frame::none()
                .stroke(egui::Stroke::new(1.0, theme::HAIRLINE))
                .rounding(theme::ROW / 2.0)
                .inner_margin(egui::Margin::symmetric(AIR.x, AIR.y))
                .show(ui, |ui| status::paint(ui, run, &words))
                .inner;
            if let Some(Door::Drop(selector)) = run.door {
                clicks.seats.push((selector, stood.1));
                clicks.picked = stood.0.then_some(selector).or(clicks.picked);
            }
        }
        if let Some(send) = &self.send {
            wrap(ui, theme::ROW);
            let seat = theme::paint::send(ui, send.live, send.help);
            crate::ui::act::tag(&seat, send.ops);
            clicks.sent = seat.clicked();
        }
    }
}

/// **Spend what the pills were given**: record where each stood for its
/// dropup and open or close the one clicked. Answers whether the send door
/// was pressed.
pub fn spend(ui: &egui::Ui, model: &mut Model, clicks: Clicks) -> bool {
    for (selector, rect) in clicks.seats {
        status::dropup::anchor(ui, model, selector, rect);
    }
    if let Some(selector) = clicks.picked {
        status::dropup::toggle(ui, model, selector);
    }
    clicks.sent
}
