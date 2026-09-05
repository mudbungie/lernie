//! The three envelopes, and the one field that makes each what it is.

use super::{deliver, fan, retire};
use crate::reply::start::Prepared;
use serde_json::json;

fn staged() -> Prepared {
    Prepared {
        workspace: "there".to_owned(),
        goal: String::new(),
        body: json!({"binding": null, "goal": "", "lineage": null,
                     "origin": "world", "workspace": "there"}),
    }
}

/// **The fan hands the prepared body back verbatim, re-addressed** — the same
/// rewrite `prompt` performs, and for the same reason: the body came back in
/// the host's spelling and this box's mapping is spent at the channel.
#[test]
fn the_fan_carries_the_prepared_body_re_addressed() {
    let built = fan(
        &staged(),
        "here".to_owned(),
        "bl-1".to_owned(),
        "p".to_owned(),
        3,
    );
    assert_eq!(built["op"], "fan");
    assert_eq!(built["n"], 3);
    assert_eq!(built["ball"], "bl-1");
    assert_eq!(built["project"], "p");
    assert_eq!(built["prepared"]["workspace"], "here");
    assert_eq!(
        built["prepared"]["origin"], "world",
        "and nothing else moved"
    );
}

/// **A candidate is addressed by its obligation and its handle**, both off the
/// work-diff row that named it — and the summary is the whole tail, verbatim.
#[test]
fn a_candidate_is_addressed_by_its_obligation_and_its_handle() {
    assert_eq!(
        deliver(
            "bl-1".to_owned(),
            "p".to_owned(),
            "at-1".to_owned(),
            "take the winner — spaces stay".to_owned()
        ),
        json!({"op": "deliver", "ball": "bl-1", "project": "p", "handle": "at-1",
               "summary": "take the winner — spaces stay"})
    );
    assert_eq!(
        retire("bl-1".to_owned(), "p".to_owned(), "at-1".to_owned()),
        json!({"op": "retire", "ball": "bl-1", "project": "p", "handle": "at-1"})
    );
}

/// **None of the three names a workspace**, which is exactly why the window
/// addresses them down a channel and argv has no row for them.
#[test]
fn none_of_the_three_names_a_workspace_at_top_level() {
    for built in [
        deliver(
            "b".to_owned(),
            "p".to_owned(),
            "h".to_owned(),
            "s".to_owned(),
        ),
        retire("b".to_owned(), "p".to_owned(), "h".to_owned()),
    ] {
        assert_eq!(crate::envelope::workspace(&built), None);
    }
    assert_eq!(
        crate::envelope::workspace(&fan(
            &staged(),
            "here".to_owned(),
            "b".to_owned(),
            "p".to_owned(),
            2
        )),
        Some("here".to_owned()),
        "the fan routes by the name inside its prepared body"
    );
}
