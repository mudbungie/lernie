//! The deletion as a value: the name the envelope is armed with, the
//! confirmation's two answers, and the one envelope both doors compose.

use crate::test_support::window::seated;
use crate::ui::Model;

/// **The envelope is `/delete`'s**: `delete-agent` on the aimed workspace, armed
/// by the listing's name for the row — the id where the listing has no row —
/// and the confirmation is put down by either answer; nothing is composed
/// with no workspace aimed.
#[test]
fn the_act_is_delete_agent_armed_by_the_name_and_either_answer_puts_the_question_down() {
    let mut model = seated();
    let id = "20260830T051200Z-a1b2";
    assert_eq!(model.name_of(id), "port the paint probe");
    assert_eq!(model.name_of("nobody"), "nobody", "the id where no row is");
    model.begin_deleting(id);
    assert_eq!(model.deleting.as_deref(), Some(id));
    model.spare();
    assert_eq!(model.deleting, None);
    assert!(model.outbox.is_empty(), "keeping sends nothing");
    model.begin_deleting(id);
    model.post_delete(id);
    assert_eq!(model.deleting, None);
    let sent = model.outbox.pop().expect("the delete").envelope;
    assert_eq!(
        sent,
        crate::verbs::delete_agent(
            "home".to_owned(),
            id.to_owned(),
            "port the paint probe".to_owned()
        )
    );
    let mut unaimed = Model::default();
    unaimed.begin_deleting(id);
    unaimed.post_delete(id);
    assert_eq!(unaimed.deleting, None);
    assert!(unaimed.outbox.is_empty(), "no workspace, no envelope");
}

/// **With the `confirm` knob off the door deletes at once** (DESIGN §4.46
/// item 4): no confirmation stands, and the same envelope goes.
#[test]
fn confirm_off_deletes_at_once() {
    let mut model = seated();
    model.settings.set(crate::settings::Toggle::Confirm, false);
    let id = "20260830T051200Z-a1b2";
    model.begin_deleting(id);
    assert_eq!(model.deleting, None, "nothing asked");
    let sent = model.outbox.pop().expect("the delete").envelope;
    assert_eq!(
        sent,
        crate::verbs::delete_agent(
            "home".to_owned(),
            id.to_owned(),
            "port the paint probe".to_owned()
        )
    );
}
