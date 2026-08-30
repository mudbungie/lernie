//! The three acts, and that each clears exactly what it invalidated.

use crate::test_support::window::{conv, seated};
use crate::ui::{Model, Notice};

/// **Aiming retires everything that was about the old wall** — the list, the
/// selection, the transcript and the tail — and touches nothing else. In
/// particular not the draft: what an operator typed is theirs until they send
/// it.
#[test]
fn aiming_at_another_wall_retires_what_was_about_the_last_one() {
    let mut model = Model {
        draft: "not sent yet".to_owned(),
        ..seated()
    };
    model.convs = vec![conv("a", "one")];
    model.aim_at("elsewhere", "theirs");
    let aim = model.aim.as_ref().expect("an aim");
    assert_eq!(
        (aim.channel.as_str(), aim.address.as_str()),
        ("elsewhere", "theirs")
    );
    assert!(model.convs.is_empty());
    assert_eq!(model.conversation, None);
    assert!(model.transcript.entries.is_empty());
    assert_eq!(model.live, None);
    assert_eq!(model.draft, "not sent yet");
}

/// Selecting retires the transcript and the tail, because they are the other
/// conversation's — and leaves the list and the aim alone, because they are not.
#[test]
fn selecting_a_conversation_retires_only_what_belonged_to_the_last_one() {
    let mut model = seated();
    model.select("another");
    assert_eq!(model.conversation.as_deref(), Some("another"));
    assert!(model.transcript.entries.is_empty());
    assert_eq!(model.live, None);
    assert_eq!(
        model.convs.len(),
        1,
        "the list is the wall's, not the row's"
    );
    assert!(model.aim.is_some());
}

/// A notice can be put down, and putting one down is all it does.
#[test]
fn dismissing_puts_the_notice_down_and_changes_nothing_else() {
    let mut model = Model {
        notice: Some(Notice::Refused("no".to_owned())),
        ..seated()
    };
    model.dismiss();
    assert_eq!(model.notice, None);
    assert!(model.aim.is_some() && model.conversation.is_some());
}
