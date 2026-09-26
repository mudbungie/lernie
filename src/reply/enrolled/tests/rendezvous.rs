//! **The rendezvous pair** (yog's `docs/REMOTE.md` §8.4, edition 20; yog
//! bl-9043): read off the engine's own fixture, re-said byte for byte, refused
//! by halves, and still inside the symbol this seat draws.

use serde_json::{Map, Value};

use super::super::{Enrolled, Handoff, PUBLIC, SALT, enrolled};
use crate::qr::Symbol;
use crate::test_support::corpus::{fixture, root};

/// The engine's own frames for this kind, vendored: a loopback engine's first
/// and a rendezvous-holding engine's second.
fn frames() -> Vec<Map<String, Value>> {
    fixture(&root().join("answers").join("enrolled.json"))
        .frames
        .into_iter()
        .map(|frame| frame.as_object().cloned().expect("an object"))
        .collect()
}

/// One frame of [`frames`], read.
fn read_frame(at: usize) -> Enrolled {
    enrolled(frames().get(at).expect("the fixture carries the frame")).expect("it reads")
}

/// **The engine's envelope, byte for byte, with the pair.** The literal is
/// what the engine's encoder writes for this frame — `serde_json`'s sorted key
/// order, compact, `ok` and `kind` gone — so an envelope that re-said only the
/// six (the defect) or re-ordered a key fails here in a sentence showing both.
#[test]
fn the_pair_is_read_and_re_said_exactly_as_the_engine_says_it() {
    let held = read_frame(1);
    assert_eq!(
        held.rendezvous,
        Some(Handoff {
            public: "0".repeat(64),
            salt: "1".repeat(64),
        })
    );
    assert_eq!(
        held.envelope(),
        r#"{"address":"engine.invalid:7737","ca":"-----BEGIN CERTIFICATE-----\nnotreal\n-----END CERTIFICATE-----\n","cert":"-----BEGIN CERTIFICATE-----\nnotreal\n-----END CERTIFICATE-----\n","grade":"foot","key":"-----BEGIN notreal KEY-----\nnotreal\n-----END notreal KEY-----\n","name":"builder","pairing_salt":"1111111111111111111111111111111111111111111111111111111111111111","rendezvous_pub":"0000000000000000000000000000000000000000000000000000000000000000","yog-enroll":1}"#
    );
}

/// **A loopback engine sends neither, and the envelope carries neither** —
/// the six it always carried, byte for byte.
#[test]
fn a_loopback_enrollment_has_no_pair_and_the_envelope_says_none() {
    let held = read_frame(0);
    assert_eq!(held.rendezvous, None);
    assert_eq!(
        held.envelope(),
        r#"{"address":"engine.invalid:7737","ca":"-----BEGIN CERTIFICATE-----\nnotreal\n-----END CERTIFICATE-----\n","cert":"-----BEGIN CERTIFICATE-----\nnotreal\n-----END CERTIFICATE-----\n","grade":"operator","key":"-----BEGIN notreal KEY-----\nnotreal\n-----END notreal KEY-----\n","name":"phone-1","yog-enroll":1}"#
    );
}

/// **Half a pair refuses, naming both.** Half a pairing derives nothing a
/// device could use, and a symbol carrying it would scan into a phone that
/// could not rove and had no way to say why.
#[test]
fn half_a_pair_refuses_and_names_both_keys() {
    for dropped in [PUBLIC, SALT] {
        let mut half = frames().swap_remove(1);
        half.remove(dropped);
        let said = enrolled(&half).expect_err("half a pair");
        for named in [PUBLIC, SALT] {
            assert!(said.contains(named), "{named} went unnamed: {said}");
        }
    }
}

/// **The caption stays material-free** with the pair aboard: the salt is
/// shared with the engine, and a scrollback keeps what it is shown.
#[test]
fn the_caption_says_nothing_of_the_pair() {
    let said = read_frame(1).caption();
    for secret in ["0000", "1111"] {
        assert!(!said.contains(secret), "{secret} is in the caption: {said}");
    }
}

/// A PEM of `len` bytes, the banner around filler lines of 64.
fn pem(banner: &str, len: usize) -> String {
    let head = format!("-----BEGIN {banner}-----\n");
    let tail = format!("-----END {banner}-----\n");
    let mut body = String::new();
    let filler: Vec<char> = "notreal".chars().collect();
    let mut at = 0;
    while head.len() + body.len() + tail.len() < len {
        let column = body.len() % 65;
        if column == 64 {
            body.push('\n');
        } else {
            body.push(filler.get(at % filler.len()).copied().unwrap_or('x'));
            at += 1;
        }
    }
    format!("{head}{body}{tail}")
}

/// **The symbol still holds it, at level M.** REMOTE §8.4 measures a real
/// mint's envelope at ~1730 bytes with the pair (`ca` 570, `cert` 623, `key`
/// 241, a foot subject). A version-40 symbol carries 1663 bytes at level Q, so
/// the pair pushed the envelope out of Q — this encoder emits M only, whose
/// 2331 still holds it. Built at the measured sizes, so a drift toward either
/// ceiling reads here rather than in a photograph.
#[test]
fn a_real_sized_envelope_with_the_pair_is_past_level_q_and_inside_level_m() {
    let held = Enrolled {
        grade: "foot".to_owned(),
        name: "phone-1".to_owned(),
        address: "engine.invalid:7737".to_owned(),
        ca: pem("CERTIFICATE", 570),
        cert: pem("CERTIFICATE", 623),
        key: pem("notreal KEY", 241),
        rendezvous: Some(Handoff {
            public: "0".repeat(64),
            salt: "1".repeat(64),
        }),
    };
    let envelope = held.envelope();
    let len = envelope.len();
    assert!((1700..1760).contains(&len), "not the measured ~1730: {len}");
    assert!(len > 1663, "it would still fit level Q: {len}");
    assert!(
        Symbol::encode(envelope.as_bytes()).is_ok(),
        "level M: {len}"
    );
}
