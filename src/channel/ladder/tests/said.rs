//! Every line a climb says, read off the bench's captured sink: one test per
//! arm, the exact words, and a repeated climb said once.

use super::*;

const P: &str = "lernie: rendezvous:";

/// The nonce and sequence of the call the seat wrote, off the inbox.
fn written(b: &Bench) -> (u64, i64) {
    let inbox = pairing().inbox_keypair().unwrap().public();
    let seq = b.node.items().iter().find(|i| i.key == inbox).unwrap().seq;
    (b.call().unwrap().nonce, seq)
}

fn refused() -> String {
    format!("{P} direct rung refused — connection refused")
}

#[test]
fn a_rendezvous_says_every_step_then_the_held_line_is_said_once() {
    let (listener, at) = listener();
    let b = Bench::new(dead(), Some(presence(at)));
    let script = vec![Reply::yes(), Reply::yes(), Reply::yes()];
    let _engine = Engine::listen(b.scratch.path(), listener, script);
    let channel = b.channel();
    assert!(channel.ask(&request(1)).is_ok());
    let (nonce, seq) = written(&b);
    assert_eq!(
        b.said(),
        [
            refused(),
            format!("{P} presence read — seq 1, 1 endpoint(s) (1 v4)"),
            format!("{P} call nonce {nonce} written — seq {seq}, 1 endpoint(s) (1 v4), 1 ack(s)"),
            format!("{P} punch for nonce {nonce} started — 1 endpoint(s) (1 v4), window 3s"),
            format!("{P} punch for nonce {nonce} landed (1 v4)"),
            format!("{P} punched line held between asks"),
        ]
    );
    assert!(channel.ask(&request(2)).is_ok());
    assert_eq!(b.said(), [format!("{P} held line taken up — no dial")]);
    assert!(channel.ask(&request(3)).is_ok());
    assert_eq!(
        b.said(),
        Vec::<String>::new(),
        "the same climb is said once"
    );
}

#[test]
fn a_direct_answer_is_said_once_and_an_entry_without_material_says_nothing() {
    let (listener, at) = listener();
    let b = Bench::new(at, None);
    let _engine = Engine::listen(b.scratch.path(), listener, vec![Reply::yes(); 2]);
    let channel = b.channel();
    assert!(channel.ask(&request(1)).is_ok());
    assert!(channel.ask(&request(2)).is_ok());
    assert_eq!(b.said(), [format!("{P} direct rung answered")]);

    let bare = Scratch::new();
    mint::material(bare.path());
    std::fs::write(bare.join(ADDRESS), dead().to_string()).unwrap();
    let material = read_dir(bare.path(), Whose::Own).unwrap().unwrap();
    let channel = Channel::open(&material)
        .unwrap()
        .tuned(b.roving(), b.clock.arc());
    assert!(channel.ask(&request(1)).is_err());
    assert_eq!(b.said(), Vec::<String>::new());
}

#[test]
fn a_held_line_found_gone_is_said_with_why_and_the_re_punch_with_how() {
    for (fate, advance, why) in [
        (Fate::Stay, GONE, "past the silence bound"),
        (Fate::Fin, Duration::ZERO, "closed at the far end"),
    ] {
        let (listener, at) = listener();
        let b = Bench::new(dead(), Some(presence(at)));
        let script = vec![Reply::yes().then(fate), Reply::yes()];
        let _engine = Engine::listen(b.scratch.path(), listener, script);
        let channel = b.channel();
        assert!(channel.ask(&request(1)).is_ok());
        b.said();
        std::thread::sleep(Duration::from_millis(200));
        b.clock.advance(advance);
        assert!(channel.ask(&request(2)).is_ok());
        assert_eq!(
            b.said(),
            [
                format!("{P} held line dropped — {why}"),
                refused(),
                format!("{P} re-punch started — 1 endpoint(s) (1 v4), window 3s"),
                format!("{P} re-punch landed (1 v4)"),
                format!("{P} punched line held between asks"),
            ]
        );
    }
}

#[test]
fn a_re_punch_that_expires_is_said_before_the_commons_is_asked() {
    let (listener, at) = listener();
    let b = Bench::new(dead(), Some(presence(at)));
    let script = vec![Reply::yes().then(Fate::Fin), Reply::yes()];
    let _engine = Engine::listen(b.scratch.path(), listener, script);
    let channel = b.channel().tuned(
        Roving {
            window: Duration::from_millis(500),
            ..b.roving()
        },
        b.clock.arc(),
    );
    assert!(channel.ask(&request(1)).is_ok());
    crate::state::worked(&b.scratch.path().display().to_string(), |w| {
        w.endpoints = vec![dead()];
    });
    b.said();
    std::thread::sleep(Duration::from_millis(200));
    assert!(channel.ask(&request(2)).is_ok());
    let said = b.said();
    assert_eq!(
        said.get(..4).unwrap(),
        [
            format!("{P} held line dropped — closed at the far end"),
            refused(),
            format!("{P} re-punch started — 1 endpoint(s) (1 v4), window 500ms"),
            format!("{P} re-punch expired after 500ms with no stream"),
        ]
    );
    assert_eq!(
        said.get(4).unwrap(),
        &format!("{P} presence read — seq 1, 1 endpoint(s) (1 v4)")
    );
}

#[test]
fn pings_discarded_are_said_with_the_answer() {
    let (listener, at) = listener();
    let b = Bench::new(dead(), Some(presence(at)));
    let _engine = Engine::listen(b.scratch.path(), listener, vec![Reply::yes().pinged(2)]);
    assert!(b.channel().ask(&request(1)).is_ok());
    assert_eq!(
        b.said().get(5..).unwrap(),
        [
            format!("{P} punched line held between asks"),
            format!("{P} held line ping discarded"),
        ]
    );
}

#[test]
fn every_way_the_commons_can_disappoint_is_said_without_a_reason_that_names_nodes() {
    let sealed_elsewhere = Keypair::from_seed([1u8; 32])
        .unwrap()
        .sign(
            pairing().presence_salt(),
            3,
            Presence { endpoints: vec![] }.seal(&[8u8; 32]).unwrap(),
        )
        .unwrap();
    let dark = FakeNode::bind(NodeId([2u8; 20]));
    let mut dark = dark;
    dark.serve(vec![], Mood::Silent, vec![]);
    let cases = [
        (
            None,
            Mood::Answer,
            None,
            format!("{P} no presence is published under this pairing"),
        ),
        (
            Some(sealed_elsewhere),
            Mood::Answer,
            None,
            format!("{P} presence seq 3 did not open under this pairing's seal key"),
        ),
        (
            None,
            Mood::Answer,
            Some(dark.addr.to_string()),
            format!(
                "{P} presence not read — the DHT walk failed (reason withheld: it names nodes)"
            ),
        ),
        (
            None,
            Mood::Answer,
            Some(String::new()),
            format!("{P} no bootstrap node resolved — no rendezvous"),
        ),
    ];
    for (held, mood, door, last) in cases {
        let b = Bench::seen(dead(), held, mood);
        let roving = Roving {
            bootstrap: door.map_or_else(|| b.roving().bootstrap, |d| vec![d]),
            ..b.roving()
        };
        assert!(
            b.channel()
                .tuned(roving, b.clock.arc())
                .ask(&request(1))
                .is_err()
        );
        assert_eq!(b.said(), [refused(), last]);
    }
}

#[test]
fn a_call_no_node_stored_and_a_punch_nothing_answered_are_said() {
    let b = Bench::seen(dead(), Some(presence(dead())), Mood::Mute);
    assert!(b.channel().ask(&request(1)).is_err());
    let said = b.said();
    let last = said.last().unwrap();
    assert!(
        last.starts_with(&format!("{P} call nonce "))
            && last
                .ends_with(" not written — the DHT walk failed (reason withheld: it names nodes)"),
        "{said:?}"
    );

    let b = Bench::new(dead(), Some(presence(dead())));
    let channel = b.channel().tuned(
        Roving {
            window: Duration::from_millis(300),
            ..b.roving()
        },
        b.clock.arc(),
    );
    assert!(channel.ask(&request(1)).is_err());
    let (nonce, _) = written(&b);
    assert_eq!(
        b.said().last().unwrap(),
        &format!("{P} punch for nonce {nonce} expired after 300ms with no stream")
    );
}

#[test]
fn a_call_carrying_only_overlay_addresses_is_said_before_the_window() {
    let (listener, at) = listener();
    let b = Bench::new(dead(), Some(presence(at)));
    let _engine = Engine::listen(b.scratch.path(), listener, vec![Reply::yes()]);
    let overlay = IpAddr::V4(Ipv4Addr::new(100, 64, 0, 1));
    let channel = b.channel().tuned(
        Roving {
            advertise: Some(vec![overlay]),
            ..b.roving()
        },
        b.clock.arc(),
    );
    assert!(channel.ask(&request(1)).is_ok(), "said, not refused");
    assert_eq!(
        b.said().get(2).unwrap(),
        &format!(
            "{P} every address the call carries (1 v4) is in an overlay or carrier-NAT range — an engine outside that network cannot reach it"
        )
    );
}
