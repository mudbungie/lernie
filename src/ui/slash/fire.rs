//! **Firing a word**: what each row does when it is taken (`super`).

use super::{DELETE, PLAN, REFRESH, rows};
use crate::ui::Model;

/// **Fire one word with the words after it.** Answers whether anything was
/// fired — a word this seat does not offer, or a verb without the parameter it
/// requires, fires nothing and leaves the draft standing.
pub fn fire(model: &mut Model, word: &str, rest: &str) -> bool {
    if !rows(model).iter().any(|row| row.word == word) {
        return false;
    }
    if let Some(fired) = conversation(model, word, rest) {
        return fired;
    }
    match word {
        PLAN => model.plan = !model.plan,
        REFRESH => model.refresh_roster(),
        w if w == crate::verbs::PIN.word => model.post_pin(true),
        w if w == crate::verbs::UNPIN.word => model.post_pin(false),
        w if w == crate::ui::enroll::WORD => model.begin_enrollment(),
        w if w == crate::ui::tuning::WORD || w == crate::ui::tuning::MODEL => {
            model.begin_tuning();
        }
        w if w == crate::ui::login::WORD => model.begin_login(),
        w if w == crate::ui::clients::WORD => model.begin_clients(),
        w if w == crate::ui::config::WORD => model.begin_configuring(),
        w if w == crate::ui::fleet::WORD => model.begin_fleet(),
        w if w == crate::ui::unmake::WORD => model.begin_unmaking(),
        w if w == crate::ui::queue::WORD => model.begin_queue(),
        w if w == crate::ui::commands::WORD => model.begin_commands(),
        w if w == crate::ui::doctor::WORD => model.begin_doctor(),
        w if w == crate::ui::trail::WORD => model.begin_trail(),
        w if w == crate::ui::board::WORD => model.begin_board(),
        w if w == crate::ui::prices::WORD => model.begin_prices(),
        w if w == crate::ui::switcher::WORD => model.switching = true,
        _ => {
            // `find`: the pane, and the needle where one was typed — spent
            // the way the pane's own control spends it, with the box not
            // cleared (`crate::ui::model::window`).
            model.begin_finding();
            if !rest.is_empty() {
                rest.clone_into(&mut model.needle);
                model.post_search();
            }
        }
    }
    true
}

/// The verbs whose subject is the selected conversation, or `None` where the
/// word is not one of them.
fn conversation(model: &mut Model, word: &str, rest: &str) -> Option<bool> {
    use crate::verbs;
    let (aim, agent) = (model.aim.clone()?, model.conversation.clone()?);
    let (address, agent) = (aim.address, agent);
    let gesture = match word {
        w if w == crate::ui::records::WORD => {
            model.begin_records();
            return Some(true);
        }
        w if w == verbs::INTERRUPT.word => {
            if rest.is_empty() {
                return Some(false);
            }
            verbs::interrupt(address, agent, rest.to_owned())
        }
        w if w == verbs::STOP.word => verbs::stop(address, agent, false),
        w if w == verbs::NUDGE.word => verbs::nudge(address, agent),
        w if w == verbs::REVOKE.word => verbs::revoke(address, agent),
        w if w == verbs::RESTORE.word => verbs::restore(address, agent),
        w if w == verbs::RETARGET.word => verbs::retarget(address, agent),
        w if w == verbs::FLAG.word => {
            if rest.is_empty() {
                return Some(false);
            }
            verbs::flag(address, agent, rest.to_owned())
        }
        DELETE => verbs::delete_agent(address, agent, rest.to_owned()),
        _ => return None,
    };
    model.outbox.push(crate::ui::Posted::act(gesture));
    Some(true)
}
