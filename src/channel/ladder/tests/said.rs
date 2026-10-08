//! Every line a climb says, read off the bench's captured sink: one test per
//! arm, the exact words, and a repeated climb said once.

use super::*;

pub(super) const P: &str = "lernie: rendezvous:";

/// The nonce and sequence of the call the seat wrote, off the inbox.
pub(super) fn written(b: &Bench) -> (u64, i64) {
    let inbox = pairing().inbox_keypair().unwrap().public();
    let seq = b.node.items().iter().find(|i| i.key == inbox).unwrap().seq;
    (b.call().unwrap().nonce, seq)
}

pub(super) fn refused() -> String {
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
            format!("{P} punch for nonce {nonce} started — 1 endpoint(s) (1 v4), window 240s"),
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
        // The dark door is silent, so its deadline is waited out: short, and
        // a floor, because nothing in that walk answers at all.
        let dark = door.as_ref().is_some_and(|d| !d.is_empty());
        let roving = Roving {
            bootstrap: door.map_or_else(|| b.roving().bootstrap, |d| vec![d]),
            dht: if dark { quick() } else { b.roving().dht },
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
    // The holder is silent to `put` and its deadline is waited out — the one
    // walk here that is a bet as well, because the same holder must answer
    // `get` inside it: 2 s is room for a loaded box (bl-73f2).
    let mute = Roving {
        dht: Config {
            deadline: Duration::from_secs(2),
            ..quick()
        },
        ..b.roving()
    };
    assert!(
        b.channel()
            .tuned(mute, b.clock.arc())
            .ask(&request(1))
            .is_err()
    );
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
