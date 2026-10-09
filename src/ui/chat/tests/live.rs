//! The live fold: a newer reading of the committed read's `Streaming` entry,
//! painted in its place and nowhere else (bl-f6b5).

use super::super::{LIVE, rows};
use crate::reply::stream::{Delta, Stream};
use crate::reply::transcript::{Block, Entry, EntryKind, Transcript, Usage};
use crate::reply::{Read, Reply};
use crate::test_support::window::said;
use crate::ui::{Channel, Model};

/// The fold the lane would hold after `text` had streamed.
fn fold(text: &str) -> Stream {
    Stream {
        text: Some(text.to_owned()),
        thinking: None,
        last_delta: Some(Delta::Text),
        tools: Vec::new(),
    }
}

/// The committed read with the turn still open: the deposit, then the
/// engine's streaming entry riding the open response.
fn in_flight(text: &str) -> Transcript {
    Transcript {
        entries: vec![
            said("op", "port it"),
            Entry {
                name: LIVE.to_owned(),
                raw: text.to_owned(),
                kind: EntryKind::Streaming {
                    thinking: String::new(),
                    text: text.to_owned(),
                },
            },
        ],
    }
}

/// **The newest fold wins.** The tail reaches a seat by two routes at two
/// cadences, and *replace* is the only reconciliation either needs — appending
/// would paint the answer twice.
#[test]
fn a_live_fold_replaces_the_streaming_entry_rather_than_standing_beside_it() {
    let committed = in_flight("half a");
    let shown = rows(&committed, Some(&fold("half a sentence")));
    assert_eq!(shown.len(), 2, "{shown:?}");
    assert_eq!(shown[1].said, "half a sentence");
    // With no newer fold the committed one still paints: the pull read's own.
    assert_eq!(rows(&committed, None)[1].said, "half a");
}

/// **Once the turn commits, the answer is painted once** (bl-f6b5). The lane's
/// last fold outlives the turn in `Model::live` — nothing clears it on a
/// commit — and a read with no `Streaming` entry is the engine saying the
/// response closed, so the fold has nothing to be newer than. Driven through
/// the model's own door, in the order the glass saw it: the fold, then the
/// committed read.
#[test]
fn a_committed_turn_retires_the_fold_that_was_folding_it() {
    let channel = Channel::default();
    let mut model = Model::default();
    model.absorb(&channel, Read::Answer(Reply::Transcript(in_flight("po"))));
    model.absorb(&channel, Read::Answer(Reply::Follow(fold("pong"))));
    let flying = rows(&model.transcript, model.live.as_ref());
    assert_eq!(flying[1].said, "pong", "{flying:?}");

    let committed = Transcript {
        entries: vec![
            said("op", "port it"),
            Entry {
                name: "002-model-a.json".to_owned(),
                raw: "{}".to_owned(),
                kind: EntryKind::Model {
                    model_id: "model-a".to_owned(),
                    blocks: vec![Block::Text("pong".to_owned())],
                    usage: Usage::new(),
                },
            },
        ],
    };
    model.absorb(&channel, Read::Answer(Reply::Transcript(committed)));
    assert!(model.live.is_some(), "the fold still stands in the model");
    let shown = rows(&model.transcript, model.live.as_ref());
    let answers: Vec<&str> = shown
        .iter()
        .filter(|row| row.said == "pong")
        .map(|row| row.who.as_str())
        .collect();
    assert_eq!(answers, vec!["model-a"], "{shown:?}");
    assert!(!shown.iter().any(|row| row.who.contains(LIVE)), "{shown:?}");
}
