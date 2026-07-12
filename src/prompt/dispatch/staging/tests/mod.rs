//! Unit tests for the transcript writer's staging sink (ARCH §2.3).
//!
//! The file is parsed back as `Vec<Content>` rather than compared as
//! raw bytes, so the assertions ride brazen's canonical vocabulary and
//! cannot drift from its serialization. Block-content behavior lives
//! here; segment-authority behavior (truncate / accumulate / seal) lives
//! in [`authority`].

mod authority;

use super::*;
use brazen::{ContentKind, Delta, Event};
use serde_json::json;
use tempfile::TempDir;

/// Event constructors shared across both test modules.
fn start(index: u32, kind: ContentKind) -> Event {
    Event::ContentStart { index, kind }
}
fn delta(index: u32, delta: Delta) -> Event {
    Event::ContentDelta { index, delta }
}
fn stop(index: u32) -> Event {
    Event::ContentStop { index }
}

/// Feed a whole segment's events (already segment-bounded) into `w`.
fn feed_all(w: &mut StagingWriter, events: &[Event]) {
    for e in events {
        w.feed(e).unwrap();
    }
}

/// Seal `w` and parse the staging file back as a canonical block list.
fn sealed_blocks(w: StagingWriter, path: &std::path::Path) -> Vec<Content> {
    w.seal().unwrap();
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

fn new_writer(dir: &TempDir) -> (StagingWriter, std::path::PathBuf) {
    let path = dir.path().join("staging.json");
    (StagingWriter::create(&path).unwrap(), path)
}

#[test]
fn appends_a_text_block_and_seals_a_one_element_array() {
    let dir = TempDir::new().unwrap();
    let (mut w, path) = new_writer(&dir);
    w.begin_segment();
    feed_all(
        &mut w,
        &[
            start(0, ContentKind::Text {}),
            delta(0, Delta::TextDelta("hi ".into())),
            delta(0, Delta::TextDelta("there".into())),
            stop(0),
        ],
    );
    assert_eq!(
        sealed_blocks(w, &path),
        vec![Content::Text("hi there".into())]
    );
}

#[test]
fn appends_text_thinking_and_tool_use_blocks_in_order() {
    let dir = TempDir::new().unwrap();
    let (mut w, path) = new_writer(&dir);
    w.begin_segment();
    feed_all(
        &mut w,
        &[
            start(0, ContentKind::Text {}),
            delta(0, Delta::TextDelta("t".into())),
            stop(0),
            start(1, ContentKind::Thinking {}),
            delta(1, Delta::ThinkingDelta("mull".into())),
            stop(1),
            start(
                2,
                ContentKind::ToolUse {
                    id: "toolu_1".into(),
                    name: "bash".into(),
                },
            ),
            delta(2, Delta::JsonDelta(r#"{"cmd":"ls"}"#.into())),
            stop(2),
        ],
    );
    assert_eq!(
        sealed_blocks(w, &path),
        vec![
            Content::Text("t".into()),
            // The v=1 stream carries no signature (§4.4) → None.
            Content::Thinking {
                text: "mull".into(),
                signature: None,
            },
            Content::ToolUse {
                id: "toolu_1".into(),
                name: "bash".into(),
                input: json!({"cmd": "ls"}),
            },
        ]
    );
}

#[test]
fn empty_tool_use_json_seals_an_empty_object_input() {
    let dir = TempDir::new().unwrap();
    let (mut w, path) = new_writer(&dir);
    w.begin_segment();
    feed_all(
        &mut w,
        &[
            start(
                0,
                ContentKind::ToolUse {
                    id: "t".into(),
                    name: "noop".into(),
                },
            ),
            stop(0),
        ],
    );
    assert_eq!(
        sealed_blocks(w, &path),
        vec![Content::ToolUse {
            id: "t".into(),
            name: "noop".into(),
            input: json!({}),
        }]
    );
}

#[test]
fn no_content_seals_an_empty_array() {
    let dir = TempDir::new().unwrap();
    let (mut w, path) = new_writer(&dir);
    w.begin_segment();
    assert_eq!(sealed_blocks(w, &path), Vec::<Content>::new());
}

#[test]
fn redacted_and_forward_compat_kinds_contribute_nothing() {
    let dir = TempDir::new().unwrap();
    let (mut w, path) = new_writer(&dir);
    w.begin_segment();
    feed_all(
        &mut w,
        &[
            start(0, ContentKind::RedactedThinking {}),
            stop(0),
            start(1, ContentKind::Other(json!({"exotic": {}}))),
            delta(1, Delta::Other(json!({"exotic_delta": "x"}))),
            stop(1),
        ],
    );
    assert_eq!(sealed_blocks(w, &path), Vec::<Content>::new());
}

#[test]
fn malformed_tool_use_json_surfaces_adapter_json_error() {
    let dir = TempDir::new().unwrap();
    let (mut w, _path) = new_writer(&dir);
    w.begin_segment();
    w.feed(&start(
        0,
        ContentKind::ToolUse {
            id: "t".into(),
            name: "bash".into(),
        },
    ))
    .unwrap();
    w.feed(&delta(0, Delta::JsonDelta("{not json".into())))
        .unwrap();
    let err = w.feed(&stop(0)).unwrap_err();
    assert!(matches!(err, Error::AdapterJson(_)), "got {err:?}");
}

#[test]
fn staging_path_sits_beside_the_response_file() {
    let resp = std::path::Path::new("/w/steps/id/001/response.json");
    assert_eq!(
        staging_path_for(resp),
        std::path::Path::new("/w/steps/id/001/staging.json"),
    );
}
