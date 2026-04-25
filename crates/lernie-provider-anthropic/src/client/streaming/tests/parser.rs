//! SSE frame-parser unit tests driven from in-memory readers.

use super::super::*;
use crate::client::Error;
use std::io::{BufReader, Cursor, Read};

#[test]
fn truncated_stream_yields_sse_error_then_ends() {
    let body = concat!(
        "event: message_start\n",
        "data: {\"type\":\"message_start\",\"message\":{\"id\":\"x\",\"model\":\"m\",\"stop_reason\":null,\"content\":[],\"usage\":{\"input_tokens\":1,\"output_tokens\":0}}}\n",
        "\n",
        "event: content_block_delta\n",
        "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"hel",
    );
    let mut stream = EventStream::new(Cursor::new(body));
    assert!(matches!(
        stream.next(),
        Some(Ok(Event::MessageStart { .. }))
    ));
    let err = stream.next().expect("expected truncation error");
    assert!(matches!(err, Err(Error::Sse(_))), "got {err:?}");
    assert!(stream.next().is_none());
}

#[test]
fn unknown_event_type_is_preserved_as_unknown_variant() {
    let body = concat!(
        "event: brand_new_thing\n",
        "data: {\"type\":\"brand_new_thing\",\"foo\":42}\n",
        "\n",
    );
    let mut stream = EventStream::new(Cursor::new(body));
    match stream.next() {
        Some(Ok(Event::Unknown(value))) => {
            assert_eq!(value["type"], "brand_new_thing");
            assert_eq!(value["foo"], 42);
        }
        other => panic!("expected Unknown, got {other:?}"),
    }
}

#[test]
fn parser_handles_crlf_line_endings_and_comments() {
    let body = "\
:keep-alive comment\r\n\
event: ping\r\n\
data: {\"type\":\"ping\"}\r\n\
\r\n\
\r\n\
event: message_stop\r\n\
data: {\"type\":\"message_stop\"}\r\n\
\r\n";
    let events: Vec<_> = EventStream::new(Cursor::new(body)).collect();
    assert_eq!(events.len(), 2);
    assert!(matches!(events[0], Ok(Event::Ping)));
    assert!(matches!(events[1], Ok(Event::MessageStop)));
}

#[test]
fn parser_joins_multiple_data_lines_in_one_event() {
    let body = "\
event: message_stop\n\
data: {\"type\":\n\
data: \"message_stop\"}\n\
\n";
    let mut stream = EventStream::new(Cursor::new(body));
    assert!(matches!(stream.next(), Some(Ok(Event::MessageStop))));
}

#[test]
fn malformed_event_json_yields_sse_error() {
    let body = "event: message_stop\ndata: { not json\n\n";
    let mut stream = EventStream::new(Cursor::new(body));
    match stream.next() {
        Some(Err(Error::Sse(msg))) => assert!(msg.contains("malformed"), "msg={msg}"),
        other => panic!("expected Sse error, got {other:?}"),
    }
}

#[test]
fn parser_skips_stray_blank_lines_before_first_frame() {
    let body = "\n\nevent: ping\ndata: {\"type\":\"ping\"}\n\n";
    let events: Vec<_> = EventStream::new(Cursor::new(body)).collect();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], Ok(Event::Ping)));
}

#[test]
fn parser_handles_data_without_space_after_colon() {
    let body = "data:{\"type\":\"ping\"}\n\n";
    let mut stream = EventStream::new(Cursor::new(body));
    assert!(matches!(stream.next(), Some(Ok(Event::Ping))));
}

/// Reader whose `read` always returns an I/O error — forces the iterator
/// down its error-handling path.
struct FailingReader;
impl Read for FailingReader {
    fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("simulated"))
    }
}

#[test]
fn parser_surfaces_underlying_reader_error_as_sse() {
    let mut stream = EventStream::new(BufReader::new(FailingReader));
    match stream.next() {
        Some(Err(Error::Sse(msg))) => assert!(msg.contains("read error"), "msg={msg}"),
        other => panic!("expected Sse, got {other:?}"),
    }
    assert!(stream.next().is_none());
}

#[test]
fn event_clone_and_debug_are_available() {
    // Guard the derive(Debug, Clone) contract so downstream code can
    // rely on it.
    let e = Event::MessageStop;
    let _ = format!("{e:?}");
    let _ = e.clone();
}
