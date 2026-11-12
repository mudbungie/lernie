//! The one request, against a local door whose root is injected.

use super::{Answer, post, web_roots};
use crate::seat::register::url::{Offer, parse};
use crate::test_support::door::{Door, NOT_FOUND, ok};
use crate::test_support::{Scratch, mint};
use rustls::RootCertStore;
use serde_json::json;

fn door(answer: impl Fn(&str) -> Vec<u8> + Send + 'static) -> (Scratch, Door, Offer) {
    let scratch = Scratch::new();
    mint::material(scratch.path());
    let door = Door::start(scratch.path(), answer);
    let offer = parse(&door.url("ABCD2345")).expect("a registration URL");
    (scratch, door, offer)
}

/// **The request is the one REMOTE §15.1 writes out**: the path with the
/// code, the host, a JSON body with its length, and a connection that closes.
#[test]
fn the_request_is_one_post_with_a_length_and_the_answer_is_its_body() {
    let (scratch, door, offer) = door(|_| ok(&json!({"said": "yes"})));
    let answer = post(&offer, r#"{"public_key":"x"}"#, Door::roots(scratch.path()));
    assert_eq!(
        answer,
        Ok(Answer {
            status: 200,
            body: br#"{"said":"yes"}"#.to_vec()
        })
    );
    let request = door.heard();
    let (head, body) = request.split_once("\r\n\r\n").expect("a head and a body");
    let mut lines = head.lines();
    assert_eq!(lines.next(), Some("POST /yog/register/ABCD2345 HTTP/1.1"));
    let rest: Vec<&str> = lines.collect();
    assert!(rest.contains(&format!("Host: {}", offer.authority()).as_str()));
    assert!(rest.contains(&"Content-Type: application/json"));
    assert!(rest.contains(&"Content-Length: 18"));
    assert!(rest.contains(&"Connection: close"));
    assert_eq!(body, r#"{"public_key":"x"}"#);
}

/// The door's bare `404` is an answer, with its status and no body.
#[test]
fn a_refusal_is_an_answer_with_its_status() {
    let (scratch, _door, offer) = door(|_| NOT_FOUND.to_vec());
    assert_eq!(
        post(&offer, "{}", Door::roots(scratch.path())),
        Ok(Answer {
            status: 404,
            body: Vec::new()
        })
    );
}

/// An answer that states no length is whatever the peer said before it
/// closed, and bytes past a stated length are not the body.
#[test]
fn a_body_ends_at_its_stated_length_or_at_the_close() {
    let (scratch, _door, offer) = door(|_| b"HTTP/1.1 502 Bad Gateway\r\n\r\nupstream".to_vec());
    let answer = post(&offer, "{}", Door::roots(scratch.path())).expect("answered");
    assert_eq!((answer.status, answer.body), (502, b"upstream".to_vec()));
    let (scratch, _door, offer) =
        door(|_| b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\n\r\nokEXTRA".to_vec());
    let answer = post(&offer, "{}", Door::roots(scratch.path())).expect("answered");
    assert_eq!(answer.body, b"ok");
}

/// What is not HTTP at all — nothing, noise, a status that is not a number —
/// is said as that, naming the host.
#[test]
fn an_answer_that_is_not_http_is_said_as_that() {
    for noise in [
        &b""[..],
        b"hello\r\n\r\n",
        b"HTTP/1.1 abc\r\n\r\n",
        b"HTTP/1.1 200",
    ] {
        let (scratch, _door, offer) = door(move |_| noise.to_vec());
        let refusal = post(&offer, "{}", Door::roots(scratch.path())).expect_err("not HTTP");
        assert!(refusal.contains("did not answer as HTTP"), "{refusal}");
        assert!(refusal.contains(&offer.authority()), "{refusal}");
    }
}

/// **The read is bounded**: an answer past the limit is refused unread.
#[test]
fn an_answer_past_the_bound_is_refused() {
    let (scratch, _door, offer) = door(|_| {
        let mut flood = b"HTTP/1.1 200 OK\r\n\r\n".to_vec();
        flood.resize(super::MOST + 4096, b'x');
        flood
    });
    let refusal = post(&offer, "{}", Door::roots(scratch.path())).expect_err("too long");
    assert!(refusal.contains("answered more than"), "{refusal}");
}

/// **The certificate is verified against the roots handed in and no others**:
/// the same door under the public web's roots, or under none, is a handshake
/// that does not complete, and the door is told nothing.
#[test]
fn a_door_the_roots_do_not_cover_is_never_spoken_to() {
    for roots in [web_roots(), RootCertStore::empty()] {
        let (_scratch, _door, offer) = door(|_| ok(&json!({})));
        let refusal = post(&offer, r#"{"public_key":"x"}"#, roots).expect_err("unverified");
        assert!(
            refusal.contains("could not be spoken to over HTTPS"),
            "{refusal}"
        );
    }
}

/// The public web's roots are a real set.
#[test]
fn the_web_roots_are_not_empty() {
    assert!(web_roots().len() > 50);
}

/// A host nothing listens at, and a host that is no name, each say so.
#[test]
fn a_door_that_is_not_there_names_the_host() {
    let dead = parse("https://127.0.0.1:1/yog/register/ABCD2345").expect("a URL");
    let refusal = post(&dead, "{}", RootCertStore::empty()).expect_err("refused");
    assert!(refusal.contains("connect 127.0.0.1:1: "), "{refusal}");
    let nameless = Offer {
        host: "-".to_owned(),
        ..dead
    };
    let refusal = post(&nameless, "{}", RootCertStore::empty()).expect_err("refused");
    assert!(refusal.contains("not a server name"), "{refusal}");
}
