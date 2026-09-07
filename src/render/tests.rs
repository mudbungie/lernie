//! **The rendering, judged the way the decoder is**: over the corpus, which is
//! upstream's own frames rather than shapes this repository imagined.
//!
//! Two directions, because a renderer dies both ways. A rendering that fails to
//! answer is loud; a rendering that quietly prints the frame back is exactly
//! the defect this module was built to remove (bl-6ae7), and it would pass a
//! "did it produce output" assertion forever.

use serde_json::json;

use super::{Form, rendered, said};
use crate::test_support::corpus;

/// The shapes the corpus does not carry a fixture of, built by hand.
mod shapes;

/// The held read, whose rendering has a memory.
mod tail;

/// **Every answer upstream can emit renders, and none of them renders as
/// JSON.** The first half is the census assertion — a kind added to
/// [`crate::reply::Reply`] with no arm here does not compile, and a kind
/// upstream grew with no reading here is already caught by the decoder's own
/// replay — and the second is the defect: the CLI's only form used to be the
/// machine's.
#[test]
fn every_answer_in_the_corpus_renders_as_something_other_than_its_frame() {
    let mut seen = 0;
    for path in corpus::files("answers") {
        for frame in corpus::fixture(&path).frames {
            let said = rendered(&frame);
            assert!(
                !said.trim().is_empty(),
                "{}: rendered to nothing",
                path.display()
            );
            assert!(
                !said.trim_start().starts_with('{'),
                "{}: rendered as its own frame — {said}",
                path.display()
            );
            assert!(
                !said.contains("cannot read that answer"),
                "{}: {said}",
                path.display()
            );
            seen += 1;
        }
    }
    assert!(seen > 30, "the corpus walk found {seen} answers");
}

/// A refusal is the engine's own sentence, printed and not interpreted; bytes
/// this seat cannot read say so about ITSELF and name the act that still shows
/// them (DESIGN §4.9's rung 2).
#[test]
fn a_refusal_is_the_engines_words_and_an_unreadable_frame_names_the_escape() {
    assert_eq!(
        rendered(&json!({"ok": false, "error": "unknown workspace \"hoem\""})),
        "refused: unknown workspace \"hoem\""
    );
    let said = rendered(&json!({"ok": true, "kind": "a-kind-from-2027"}));
    assert!(said.contains("a-kind-from-2027"), "{said}");
    assert!(said.contains("--json"), "{said}");
}

/// **`--json` is the frame, byte for byte.** The boundary stays JSON; what the
/// default adds is the part you look at.
#[test]
fn the_json_form_is_the_frames_exactly_as_they_crossed() {
    let frames = vec![
        json!({"ok": true, "kind": "nudged"}),
        json!({"ok": true, "kind": "flagged"}),
    ];
    assert_eq!(
        said(&frames, Form::Json),
        "{\"kind\":\"nudged\",\"ok\":true}\n{\"kind\":\"flagged\",\"ok\":true}"
    );
    assert_eq!(
        said(&frames, Form::Rendered),
        "nudged — the advance is running\nflagged — it is on the queue now"
    );
    assert_eq!(Form::default(), Form::Rendered);
}

/// **The empty world says the next act** (bl-b00f). A fresh world answers
/// `rows: []` correctly and terminally, and the one thing a person needs at
/// that moment is that starting a conversation founds the first wall.
#[test]
fn an_empty_roster_names_the_act_that_founds_the_first_wall() {
    let said = rendered(&json!({"ok": true, "kind": "workspaces", "rows": []}));
    assert!(said.contains("lernie start home"), "{said}");
    let queue = rendered(&json!({"ok": true, "kind": "attention", "rows": []}));
    assert!(queue.contains("nothing is waiting on you"), "{queue}");
}

/// **A transcript is the entries and not their bytes.** Every entry carries a
/// `raw` beside its reading, so the machine form is each conversation twice
/// over on one line — which is the whole of why it could not be read in a
/// terminal.
#[test]
fn a_transcript_prints_what_was_said_and_not_the_raw_bytes_beside_it() {
    let frame = json!({"ok": true, "kind": "transcript", "rows": [
        {"name": "001-user.md", "raw": "---\nfrom: user\n---\nship it",
         "kind": "delivered", "sender": "user", "body": "ship it"}
    ]});
    let said = rendered(&frame);
    assert!(said.contains("001-user.md"), "{said}");
    assert!(said.contains("from user"), "{said}");
    assert!(said.contains("ship it"), "{said}");
    assert!(!said.contains("---"), "the raw bytes are printed: {said}");
}

/// **The fourth `framing` word reaches the glass as itself** (PROTOCOL 18, yog
/// bl-ab53). `in_flight` is the step being written right now, told apart from
/// the one an interrupt cut — a new VALUE and not a new key, which no field
/// signature can see. This seat carries a class token verbatim on §4.9's rung
/// 3, so what it owes is the assertion that the word arrives and is not
/// painted as a neighbour it is not: `killed` is the word that makes an
/// interrupt legible, and reading it over a healthy conversation costs the
/// vocabulary the interrupt depends on.
#[test]
fn an_in_flight_step_paints_as_itself_and_never_as_the_word_for_a_kill() {
    let said = rendered(
        &json!({"ok": true, "kind": "steps", "orphan": "none", "rows": [
            {"seq": "007", "framing": "in_flight", "attempts": 1, "wound": "none",
             "started_at": "t0", "commit": "abc",
             "tokens": {"input": 11, "output": 22, "cache_read": 33, "cache_write": 44, "total": 99}}
        ]}),
    );
    assert!(said.contains("in_flight"), "{said}");
    assert!(!said.contains("killed"), "{said}");
}

/// **What a reviewer staged for a wall's config, and one of them whole**
/// (PROTOCOL 18, yog bl-dd88). The standing crosses as the engine's own word —
/// `fresh` means the lineage still stands where the reviewer read it — and is
/// never inferred here from an empty `lineages`, which is REMOTE §9.4's rule
/// that the wire states a derivation so no seat owns one.
#[test]
fn a_proposals_listing_says_the_standing_the_engine_read_and_never_infers_one() {
    let row = |id, fresh, lineages| {
        json!({"id": id, "fresh": fresh, "lineages": lineages, "parent": "9f2c1ab4",
               "diffstat": "1 file changed, 6 insertions(+)", "subject": "notes: what it taught"})
    };
    let said = rendered(
        &json!({"ok": true, "kind": "proposals", "whole": "commit 71011c3d",
        "rows": [row("r-1", true, json!(["default"])), row("r-2", false, json!([]))]}),
    );
    assert!(said.contains("r-1"), "{said}");
    assert!(said.contains("fresh"), "{said}");
    assert!(said.contains("default"), "{said}");
    assert!(said.contains("stale"), "{said}");
    assert!(
        said.contains("no lineage still heads at its parent"),
        "an empty lineage list is the stale case said out loud: {said}"
    );
    assert!(said.contains("commit 71011c3d"), "{said}");
    assert!(
        rendered(&json!({"ok": true, "kind": "proposals", "rows": []}))
            .contains("nothing is staged for this wall"),
        "nothing staged is an empty answer, not a refusal"
    );
}
