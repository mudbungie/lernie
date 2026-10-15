//! A sent message between frames: recorded by the deposit, acknowledged by the
//! receipt, retired by the read that carries it, and given back by a refusal
//! or an act that never left the box.

use super::{DEPOSITS, Sent};
use crate::channel::Reach;
use crate::reply::transcript::Transcript;
use crate::reply::{Outcome, Read, Reply};
use crate::test_support::window::{own, said, seated};
use crate::ui::Model;

const HERE: &str = "20260830T051200Z-a1b2";
const THERE: &str = "20260830T052000Z-b2c3";

/// A deposit's receipt, exiting as `exit`.
fn ran(exit: i32) -> Read {
    Read::Answer(Reply::Outcome(Outcome {
        exit,
        stdout: String::new(),
        stderr: String::new(),
    }))
}

/// A model that sent `words` to the selected conversation.
fn having_sent(words: &str) -> Model {
    let mut model = seated();
    model.deposit(
        crate::verbs::message,
        "home".to_owned(),
        HERE.to_owned(),
        words.to_owned(),
    );
    model
}

/// **The deposit records what it composed beside the act**: the
/// conversation, the words and the committed length, unacknowledged — both
/// doors, so the slash line's `/interrupt` is on the glass as the box's
/// message is.
#[test]
fn a_deposit_is_recorded_the_frame_it_is_composed() {
    let mut model = having_sent("ship it");
    assert_eq!(
        model.sent,
        vec![Sent {
            conversation: HERE.to_owned(),
            words: "ship it".to_owned(),
            after: 1,
            acked: false,
        }]
    );
    assert_eq!(model.outbox.len(), 1);
    assert!(crate::ui::slash::fire(&mut model, "interrupt", "no, this"));
    assert_eq!(model.sent[1].words, "no, this");
    assert_eq!(model.outbox[1].envelope["op"], "interrupt");
    assert!(DEPOSITS.contains(&"interrupt"));
}

/// **An `ok` receipt is the ink and nothing else**: the oldest unacknowledged
/// record is marked, the row stands, and a receipt of any other op leaves
/// every record alone.
#[test]
fn an_ok_receipt_acknowledges_the_oldest_record_and_retires_nothing() {
    let mut model = having_sent("one");
    model.deposit(
        crate::verbs::message,
        "home".to_owned(),
        HERE.to_owned(),
        "two".to_owned(),
    );
    let flat = own().channel;
    model.receipt(&flat, crate::verbs::NUDGE.word, ran(0));
    assert!(model.sent.iter().all(|sent| !sent.acked));
    model.receipt(&flat, crate::verbs::MESSAGE.word, ran(0));
    assert_eq!(
        model.sent.iter().map(|sent| sent.acked).collect::<Vec<_>>(),
        vec![true, false]
    );
    model.receipt(&flat, crate::verbs::INTERRUPT.word, ran(1));
    assert!(!model.sent[1].acked, "a failed run is the bar's");
    model.receipt(&flat, crate::verbs::INTERRUPT.word, ran(0));
    assert!(model.sent.iter().all(|sent| sent.acked));
    model.receipt(&flat, crate::verbs::MESSAGE.word, ran(0));
    assert_eq!(model.sent.len(), 2, "a receipt with nothing left to answer");
}

/// **A refusal drops the record and the words go back in the box** — ahead of
/// whatever has been typed since, with a line break, so neither is lost.
#[test]
fn a_refusal_gives_the_words_back_ahead_of_the_draft() {
    let flat = own().channel;
    let mut model = having_sent("ship it");
    model.receipt(
        &flat,
        crate::verbs::MESSAGE.word,
        Read::Refusal("no".to_owned()),
    );
    assert!(model.sent.is_empty());
    assert_eq!(model.draft, "ship it");
    let mut model = having_sent("ship it");
    model.draft = "and then".to_owned();
    model.receipt(
        &flat,
        crate::verbs::INTERRUPT.word,
        Read::Refusal("no".to_owned()),
    );
    assert_eq!(model.draft, "ship it\nand then");
}

/// **The read that carries the words retires the record the same frame** —
/// only an operator's delivery, only past the recorded length, only for the
/// selected conversation; a model's turn saying the same words, or an older
/// deposit of them, retires nothing.
#[test]
fn the_transcript_that_carries_the_words_retires_the_record() {
    let flat = own().channel;
    let mut model = having_sent("port it");
    let committed = |entries: Vec<crate::reply::transcript::Entry>| {
        Read::Answer(Reply::Transcript(Transcript { entries }))
    };
    model.absorb(&flat, committed(vec![said("op", "port it")]));
    assert_eq!(model.sent.len(), 1, "the one before it is not this one");
    model.absorb(
        &flat,
        committed(vec![said("op", "port it"), said("peer", "port it")]),
    );
    assert_eq!(model.sent.len(), 1, "a peer's row is not the operator's");
    model.absorb(
        &flat,
        committed(vec![said("op", "port it"), said("user", "port it")]),
    );
    assert!(model.sent.is_empty());
    assert_eq!(model.transcript.entries.len(), 2);

    let mut model = having_sent("ship it");
    model.sent.push(Sent {
        conversation: THERE.to_owned(),
        words: "ship it".to_owned(),
        after: 0,
        acked: false,
    });
    model.absorb(
        &flat,
        committed(vec![said("op", "port it"), said("op", "ship it")]),
    );
    assert_eq!(
        model
            .sent
            .iter()
            .map(|sent| sent.conversation.as_str())
            .collect::<Vec<_>>(),
        vec![THERE],
        "the read is the selected conversation's"
    );
}

/// **The records are keyed by conversation**: a tab change paints the other
/// conversation's, and none of this one's.
#[test]
fn a_record_stands_with_its_own_conversation() {
    let mut model = having_sent("ship it");
    model.select(THERE);
    assert!(model.sent_here().next().is_none());
    model.deposit(
        crate::verbs::message,
        "home".to_owned(),
        THERE.to_owned(),
        "there".to_owned(),
    );
    assert_eq!(
        model
            .sent_here()
            .map(|sent| sent.words.as_str())
            .collect::<Vec<_>>(),
        vec!["there"]
    );
    model.select(HERE);
    assert_eq!(
        model
            .sent_here()
            .map(|sent| sent.words.as_str())
            .collect::<Vec<_>>(),
        vec!["ship it"]
    );
}

/// **An act that never left this box takes its record with it**; one that
/// crossed and earned no answer keeps it, in doubt, and so does any other
/// op's failure.
#[test]
fn an_unsent_act_drops_its_record_and_an_unanswered_one_keeps_it() {
    let mut model = having_sent("ship it");
    model.acted(
        crate::verbs::NUDGE.word,
        &Reach::Unsent("connect: connection refused".to_owned()),
        None,
    );
    assert_eq!(
        model.sent.len(),
        1,
        "another op's failure is not this one's"
    );
    model.acted(
        crate::verbs::MESSAGE.word,
        &Reach::Unanswered("read: timed out".to_owned()),
        Some("ship it".to_owned()),
    );
    assert_eq!(model.sent.len(), 1, "the engine may have it");
    model.acted(
        crate::verbs::MESSAGE.word,
        &Reach::Unsent("connect: connection refused".to_owned()),
        Some("ship it".to_owned()),
    );
    assert!(model.sent.is_empty());
    assert_eq!(model.draft, "ship it", "the words are the refund's");
}
