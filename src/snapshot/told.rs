//! **Assertion (e): every painted control tells** (DESIGN §4.46 item 2).
//!
//! The ruling is that every interface element says what it does when the
//! pointer rests on it. Every painter takes that sentence beside the
//! control's word and attaches it as the toolkit's tooltip
//! (`theme::paint::told`), so a control without one does not compile; this
//! is what keeps the sentence from being attached and never shown — a seat
//! laid where egui cannot hover it, a painter that forgot to end in `told`.
//!
//! **The subject is every painted control**, the twin of `super::lit`: the
//! seats every painter recorded (`crate::ui::act::bridge`), read off the
//! same tree. The pointer is put at a control's centre exactly as `lit`
//! puts it, then **held still** while the clock steps past the style's
//! delay and its grace — frames with no pointer event, since egui shows a
//! tooltip only once the pointer has rested — and the toolkit is asked
//! whether that seat's tooltip stood on the last frame
//! (`egui::popup::was_tooltip_open_last_frame`). A seat with none is a
//! complaint naming the control.
//!
//! **What is not judged**: a field that takes text — its answer to the
//! pointer is the caret, and the sentence is its hint — and a control egui
//! does not report hovered at its centre, for `lit`'s reasons: disabled,
//! clipped, or covered, the pointer is not on it.

use crate::ui::{Model, act};
use egui_kittest::Harness;
use egui_kittest::kittest::{Queryable, by};

/// One painted control as the tree offers it.
struct Control {
    node: u64,
    label: String,
    role: String,
}

/// **Whether a role takes text**, by the tree's own name for it.
fn takes_text(role: &str) -> bool {
    role.ends_with("TextInput")
}

/// **Every painted control**, read in one pass over the settled tree.
fn controls(harness: &Harness<'_, Model>) -> Vec<Control> {
    let bridge = act::bridged(&harness.ctx);
    harness
        .query_all(by())
        .filter(|node| !node.is_hidden() && bridge.contains_key(&node.id().0))
        .map(|node| Control {
            node: node.id().0,
            label: node.label().or_else(|| node.value()).unwrap_or_default(),
            role: format!("{:?}", node.role()),
        })
        .collect()
}

/// **Hand the frame one pointer event and settle it**, as `lit` does: two
/// passes on a clock that steps past the bar's linger.
fn settle(harness: &mut Harness<'_, Model>, event: egui::Event) {
    harness.input_mut().events.push(event);
    for _ in 0..2 {
        let input = harness.input_mut();
        input.time = Some(input.time.unwrap_or(1_000.0) + f64::from(crate::ui::bar::LINGER) + 1.0);
        harness.step();
    }
}

/// **Let the pointer rest**: frames with no event on a clock stepping past
/// the delay and the grace each time, so the toolkit sees a pointer that
/// has not moved for longer than it asks.
fn rest(harness: &mut Harness<'_, Model>) {
    let wait = harness.ctx.style().interaction.tooltip_delay
        + harness.ctx.style().interaction.tooltip_grace_time
        + 1.0;
    for _ in 0..3 {
        let input = harness.input_mut();
        input.time = Some(input.time.unwrap_or(1_000.0) + f64::from(wait));
        harness.step();
    }
}

/// **Every painted control, hovered, rested on, and asked whether it told.**
/// The count is how many were judged; an empty list is the assertion
/// holding.
pub(crate) fn complaints(at: &str, harness: &mut Harness<'_, Model>) -> (usize, Vec<String>) {
    let mut judged = 0;
    let mut out = Vec::new();
    for control in controls(harness) {
        if takes_text(&control.role) {
            continue;
        }
        let id =
            act::seat_of(&harness.ctx, control.node).expect("a painted node recorded its seat");
        settle(harness, egui::Event::PointerGone);
        let mut seat = harness
            .ctx
            .read_response(id)
            .expect("a painted node is a seat egui holds");
        for _ in 0..2 {
            settle(harness, egui::Event::PointerMoved(seat.rect.center()));
            seat = harness
                .ctx
                .read_response(id)
                .expect("a seat the pointer stands on is still held");
        }
        if !seat.hovered() {
            continue;
        }
        judged += 1;
        rest(harness);
        if !egui::popup::was_tooltip_open_last_frame(&harness.ctx, id) {
            out.push(format!(
                "{at}: the {} reading {:?} says nothing with the pointer resting on it — \
                 it is painted without its help, or past where a tooltip can open",
                control.role, control.label
            ));
        }
    }
    (judged, out)
}

#[cfg(test)]
mod tests {
    use crate::ui::Model;
    use egui_kittest::HarnessBuilder;

    /// A window painting one told control and one bare one.
    fn toy(bare: bool) -> egui_kittest::Harness<'static, Model> {
        let mut harness = HarnessBuilder::<Model>::default()
            .with_size(egui::Vec2::new(200.0, 100.0))
            .build_state(
                move |ctx: &egui::Context, _: &mut Model| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let seat = crate::ui::theme::paint::button(ui, "told", true, "tells");
                        if bare {
                            // A seat recorded without its sentence: the
                            // bridge alone, which no painter does.
                            let mute = ui.add(egui::Label::new("mute").sense(egui::Sense::click()));
                            crate::ui::act::bridge(&mute);
                        }
                        let _ = seat;
                    });
                },
                Model::default(),
            );
        harness.run();
        harness
    }

    /// **The other direction**: a told control passes, and a painted seat
    /// with no sentence is named.
    #[test]
    fn a_told_control_passes_and_a_mute_one_is_named() {
        let mut harness = toy(false);
        let (judged, said) = super::complaints("toy", &mut harness);
        assert_eq!((judged, said.len()), (1, 0), "{said:?}");
        let mut harness = toy(true);
        let (judged, said) = super::complaints("toy", &mut harness);
        assert_eq!((judged, said.len()), (2, 1), "{said:?}");
        assert!(said.concat().contains("mute"), "{said:?}");
    }
}
