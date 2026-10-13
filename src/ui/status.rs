//! **The status rule**: one line between the transcript and the prompt that
//! says where you are and what it costs (DESIGN §4.42, item 2;
//! `docs/STYLE.md` §2, *the rule says where you are*).
//!
//! Left to right: `─ <state> · <age> │ <model> │ <spend> │ <N waiting> │
//! <N engines> ─ <name>`. The state is the wire's own word for the selected
//! conversation in its state's ink, `ready` in weak ink where nothing is
//! selected — and **the notice stands in that slot** in its own ink while
//! there is one: what the seat last heard that was not content is a state of
//! the window, read where the window's state is read, and Esc puts it down
//! (`Model::escape`). The segments are the conversation header's facts moved
//! onto one line: the model that answered, what the conversation has spent
//! (§4.41's words, never `$0`), how many conversations across every engine
//! are asking — in the attention accent, the one green on a resting window —
//! and how many engines this seat reaches. The right end is the selected
//! conversation's name, the aimed wall's where none is selected, and the word
//! `engines` where nothing is aimed.
//!
//! **Three of the runs are doors** (bl-4e55, bl-2cb0): the name opens the
//! switcher, the model opens the role tuning and the waiting count opens the
//! queue — each by firing the completion menu's own row ([`door`]), so a
//! click here is `/conversations`, `/model` or `/attention` typed, wears
//! that row's `act:` tags, and is a control exactly where the row is offered.
//! A wall whose transcript names no model says `no model` in weak ink in that
//! slot, so the way to choose one is on the glass before anything answered.
//!
//! **Segments drop from the right as the width runs out, the name last**:
//! a rule that wrapped would be two lines, and a rule that elided the name
//! would say everything but where you are.
//!
//! **A notice too long for the slot is never cut** (bl-7e3e; bl-3d0f ruled
//! why: the half a cut sentence loses is the remedy) **and never costs a
//! fact**. Where it does not fit beside every fact and the name, it stands
//! whole on its own lines directly above the rule, wrapped, in its ink, until
//! Esc — and the rule keeps its one line, the slot saying what it says with
//! no notice.

use crate::ui::slash::Row;
use crate::ui::{Model, theme};

/// The glyph the rule opens with, and the one that parts the facts from the
/// name.
pub const LEAD: &str = theme::glyph::LEAD;
/// What parts one fact from the next.
pub const SEP: &str = theme::glyph::SEP;
/// The state slot's word where nothing is selected and nothing is heard.
pub const READY: &str = "ready";
/// The right end where nothing is aimed.
pub const ENGINES: &str = "engines";
/// The model slot on a wall whose transcript names none and whose roles are
/// not in hand: the door to choosing one, said where the model would be. It
/// names the control and claims nothing about the wall (bl-0966): the wall's
/// config names a model this seat has simply not read, and `no model` read as
/// a misconfiguration that was not there.
pub const NO_MODEL: &str = "model…";
/// The role whose model a start runs on.
const WORKER: &str = "worker";
/// **The mode the next start is armed in**, first among the facts while
/// `/plan` holds it (§4.42 item 3; bl-6f3f): the box's hint says it only
/// until the first character is typed over it.
pub const PLAN: &str = "plan";

/// One run of the rule: its words, its ink, and the menu row it opens where
/// it is a door.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    pub text: String,
    pub ink: egui::Color32,
    pub door: Option<Row>,
}

fn run(text: impl Into<String>, ink: egui::Color32) -> Run {
    Run {
        text: text.into(),
        ink,
        door: None,
    }
}

/// **The menu row `word` names, where this seat offers it right now** — the
/// one answer to whether a run is a door, so a run opens nothing the menu
/// would not.
pub fn door(model: &Model, word: &str) -> Option<Row> {
    crate::ui::slash::rows(model)
        .into_iter()
        .find(|row| row.word == word)
}

/// **The state slot**: the notice, else the selected conversation's state
/// and age, else `ready`.
pub fn state(model: &Model) -> Run {
    if let Some(notice) = &model.notice {
        let ink = match notice {
            crate::ui::Notice::Said(_) => theme::accent(theme::State::Annotation),
            _ => theme::accent(theme::State::Error),
        };
        return run(notice.line(), ink);
    }
    calm(model)
}

/// **The state slot with no notice in it** — what it says under a notice too
/// long to stand there ([`render`]).
pub fn calm(model: &Model) -> Run {
    match selected(model) {
        Some(row) => run(
            format!(
                "{} · {}",
                row.state.label(),
                crate::ui::convs::age(row.age_secs)
            ),
            theme::state_ink(&row.state),
        ),
        None => run(READY, theme::INK_WEAK),
    }
}

/// The selected conversation's own row, where the list holds one.
fn selected(model: &Model) -> Option<crate::reply::convs::ConvRow> {
    let id = model.conversation.as_ref()?;
    model.rows().into_iter().find(|row| row.root_id == *id)
}

/// The model that last answered on the transcript, where one did.
fn answered_by(model: &Model) -> Option<String> {
    model
        .transcript
        .entries
        .iter()
        .rev()
        .find_map(|entry| match &entry.kind {
            crate::reply::transcript::EntryKind::Model { model_id, .. } => Some(model_id.clone()),
            _ => None,
        })
}

/// How many conversations are asking, across every engine this seat reaches.
pub fn waiting(model: &Model) -> u64 {
    model
        .roster
        .iter()
        .flat_map(|chunk| chunk.walls.iter())
        .map(|wall| wall.attention)
        .sum()
}

/// **The facts between the state and the name**, in order, each a run.
pub fn facts(model: &Model) -> Vec<Run> {
    let mut out = Vec::new();
    if model.plan {
        out.push(run(PLAN, theme::BRAND));
    }
    let opens = door(model, crate::ui::tuning::MODEL);
    // **The model slot**: what last answered; else the wall's worker model,
    // where its roles are in hand — what the next turn will run on; else the
    // door's own word, where there is a door.
    let worker = || {
        model
            .roles
            .as_ref()
            .and_then(|roles| roles.iter().find(|row| row.role == WORKER))
            .map(|row| row.model.clone())
    };
    if let Some(said) = answered_by(model)
        .or_else(worker)
        .or_else(|| opens.map(|_| NO_MODEL.to_owned()))
    {
        out.push(Run {
            door: opens,
            ..run(said, theme::INK_WEAK)
        });
    }
    if let Some(agent) = model.agent() {
        out.push(run(
            crate::ui::records::header::costing(agent),
            theme::INK_WEAK,
        ));
    }
    let asking = waiting(model);
    if asking > 0 {
        // **The count is the queue's door** (§4.42, item 8; bl-2cb0), and
        // the queue needs no aim, so it opens on every window that counts.
        out.push(Run {
            door: door(model, crate::ui::queue::WORD),
            ..run(
                format!("{asking} waiting"),
                theme::accent(theme::State::Attention),
            )
        });
    }
    let engines = model.roster.len();
    out.push(run(
        if engines == 1 {
            "1 engine".to_owned()
        } else {
            format!("{engines} engines")
        },
        theme::INK_WEAK,
    ));
    out
}

/// **The right end**: the conversation, the wall, or the window — and the
/// door to the switcher, wherever it stands.
pub fn name(model: &Model) -> Run {
    let text = match (selected(model), &model.aim) {
        (Some(row), _) => row.display,
        (None, Some(aim)) => aim.address.clone(),
        (None, None) => ENGINES.to_owned(),
    };
    Run {
        door: door(model, crate::ui::switcher::WORD),
        ..run(text, theme::INK)
    }
}

/// The width one run takes at the rule's size.
fn width(ui: &egui::Ui, text: &str) -> f32 {
    let font = egui::TextStyle::Small.resolve(ui.style());
    ui.fonts(|fonts| {
        fonts
            .layout_no_wrap(text.to_owned(), font, theme::INK)
            .size()
            .x
    })
}

/// **Which facts fit** beside the state and the name at this width: the
/// leading ones, dropped from the right until the line holds.
pub fn fitting(ui: &egui::Ui, state: &Run, facts: &[Run], name: &str, room: f32) -> usize {
    let gap = ui.spacing().item_spacing.x;
    let sep = width(ui, SEP) + 2.0 * gap;
    let mut used = width(ui, LEAD) * 2.0 + width(ui, &state.text) + width(ui, name) + 6.0 * gap;
    let mut shown = 0;
    for fact in facts {
        let more = sep + width(ui, &fact.text);
        if used + more > room {
            break;
        }
        used += more;
        shown += 1;
    }
    shown
}

/// **Paint one run**: a label, or a door that tints under the pointer, wears
/// its row's tags and fires its row's word. Answers the word it fired.
fn paint(ui: &mut egui::Ui, run: &Run) -> Option<&'static str> {
    let Some(row) = run.door else {
        ui.label(egui::RichText::new(&run.text).small().color(run.ink));
        return None;
    };
    let seat = theme::paint::link(ui, &run.text, run.ink);
    crate::ui::act::tag(&seat, row.ops);
    seat.clicked().then_some(row.word)
}

/// Paint the rule, and open what a click on one of its doors opens.
pub fn render(ui: &mut egui::Ui, model: &mut Model) {
    let mut state = state(model);
    let facts = facts(model);
    let name = name(model);
    let room = ui.available_width();
    // **A notice the rule cannot hold whole stands above it** (bl-7e3e): its
    // own lines, wrapped, in its ink, and the slot says what it would say
    // without it — so the remedy at the sentence's end is read, and the rule
    // still says where you are and what else is true.
    if model.notice.is_some() && fitting(ui, &state, &facts, &name.text, room) < facts.len() {
        let said = egui::RichText::new(&state.text).small().color(state.ink);
        ui.add(egui::Label::new(said).wrap());
        state = calm(model);
    }
    let shown = fitting(ui, &state, &facts, &name.text, room);
    let faint = |ui: &mut egui::Ui, text: &str| {
        ui.label(egui::RichText::new(text).small().color(theme::INK_FAINT));
    };
    let mut fired = None;
    ui.horizontal(|ui| {
        faint(ui, LEAD);
        paint(ui, &state);
        for fact in facts.iter().take(shown) {
            faint(ui, SEP);
            fired = paint(ui, fact).or(fired);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            fired = paint(ui, &name).or(fired);
            faint(ui, LEAD);
        });
    });
    if let Some(word) = fired {
        crate::ui::slash::fire(model, word, "");
    }
}

#[cfg(test)]
mod tests;
