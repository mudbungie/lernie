//! **Firing a word**: what each row does when it is taken (`super`).

use super::{DELETE, LEAD, PLAN, REFRESH, rows};
use crate::ui::Model;

/// **Fire one word with the words after it.** Answers whether anything was
/// fired — a word this seat does not offer, or a verb without the parameter it
/// requires ([`wanting`]), fires nothing and leaves the draft standing.
pub fn fire(model: &mut Model, word: &str, rest: &str) -> bool {
    if !rows(model).iter().any(|row| row.word == word) || wanting(model, word, rest).is_some() {
        return false;
    }
    // **One overlay stands at a time** (bl-8c80; `Model::uncover`): a word
    // that opens a pane opens it alone, whichever chord reached it from
    // under the switcher or another pane. Nothing fires under the
    // enrollment, which only its own gesture leaves.
    if model.enroll.is_some() {
        return false;
    }
    model.uncover();
    if let Some(fired) = conversation(model, word, rest) {
        return fired;
    }
    match word {
        PLAN => model.plan = !model.plan,
        REFRESH => model.refresh_roster(),
        w if w == crate::verbs::PIN.word => model.post_pin(true),
        w if w == crate::verbs::UNPIN.word => model.post_pin(false),
        w if w == crate::ui::enroll::WORD => model.begin_enrollment(),
        w if w == crate::ui::tuning::WORD => model.begin_tuning(),
        w if w == crate::ui::tuning::MODEL => model.drop_up(crate::ui::Selector::Model),
        w if w == crate::ui::login::WORD => model.begin_login(),
        w if w == crate::ui::clients::WORD => model.begin_clients(),
        w if w == crate::ui::config::WORD => model.begin_configuring(),
        w if w == crate::ui::fleet::WORD => model.begin_fleet(),
        w if w == crate::ui::unmake::WORD => model.begin_unmaking(),
        w if w == crate::ui::queue::WORD => model.begin_queue(),
        w if w == crate::ui::commands::WORD => model.begin_commands(),
        w if w == crate::ui::doctor::WORD => model.begin_doctor(),
        w if w == crate::ui::settings::WORD => model.lookup = Some(crate::ui::Lookup::Settings),
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
        // A deposit, recorded as sent the frame it is composed (DESIGN
        // §4.45 item 4), through the one door the box itself uses.
        w if w == verbs::INTERRUPT.word => {
            model.deposit(verbs::interrupt, address, agent, rest.to_owned());
            return Some(true);
        }
        // `children` is the one word `/stop` takes: the verb table's own
        // summary offers it, so the word typed after it is spent (bl-0740).
        w if w == verbs::STOP.word => {
            verbs::stop(address, agent, rest == verbs::conversation::CHILDREN)
        }
        w if w == verbs::NUDGE.word => verbs::nudge(address, agent),
        w if w == verbs::REVOKE.word => verbs::revoke(address, agent),
        w if w == verbs::RESTORE.word => verbs::restore(address, agent),
        w if w == verbs::RETARGET.word => verbs::retarget(address, agent),
        w if w == verbs::FLAG.word => verbs::flag(address, agent, rest.to_owned()),
        // **The one door the bar's trashcan goes through too** (bl-e837):
        // the word was held to the name by [`wanting`], so the seat types it.
        DELETE => {
            model.post_delete(&agent);
            return Some(true);
        }
        _ => return None,
    };
    model.outbox.push(crate::ui::Posted::act(gesture));
    Some(true)
}

/// **Why `word` cannot fire with `rest` after it**, in the window's words, or
/// `None` where it can (bl-df70).
///
/// A verb that takes words takes them on the same line, and one with them
/// missing fires nothing (DESIGN §4.42, item 5) — and says so, because a verb
/// that silently does nothing reads as a key that is dead.
///
/// **`/delete` is armed by the conversation's own name and by nothing else.**
/// The wire's typed name arms only the subtree: a leaf is deleted by any
/// word, an empty one included. So the seat holds the arming §4.20 asks for
/// itself — the name the engine compares (its display name, retyped,
/// surrounding space forgiven) or no envelope at all — and states the subject
/// in the refusal, as an unmaking states its subject before it offers the act.
pub fn wanting(model: &Model, word: &str, rest: &str) -> Option<String> {
    use crate::verbs;
    match word {
        w if w == verbs::FLAG.word && rest.is_empty() => Some(format!(
            "{LEAD}{w} needs its reason after it — {LEAD}{w} <reason>"
        )),
        w if w == verbs::INTERRUPT.word && rest.is_empty() => Some(format!(
            "{LEAD}{w} needs the words to say instead — {LEAD}{w} <words>"
        )),
        DELETE => {
            let name = model.name_of(model.conversation.as_deref()?);
            (rest.trim() != name).then(|| {
                format!(
                    "{LEAD}{DELETE} is armed by the conversation's own name — {LEAD}{DELETE} {name}"
                )
            })
        }
        _ => None,
    }
}
