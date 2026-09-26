//! Every rung falling through, and the sentence each leaves; and the stated
//! defaults.

use super::*;

#[test]
fn a_dark_commons_is_said_after_the_direct_refusal() {
    let mut node = FakeNode::bind(NodeId([2u8; 20]));
    node.serve(vec![], Mood::Silent, vec![]);
    let b = Bench::new(dead(), None);
    let channel = b.channel().tuned(
        Roving {
            bootstrap: vec![node.addr.to_string()],
            ..b.roving()
        },
        b.clock.arc(),
    );
    let Err(Reach::Unsent(said)) = channel.ask(&request(1)) else {
        panic!("nothing crossed");
    };
    assert!(said.starts_with("connect 127.0.0.1:"), "{said}");
    assert!(
        said.contains("; rendezvous: no DHT node answered get"),
        "{said}"
    );
}

#[test]
fn every_way_the_rendezvous_can_disappoint_is_a_sentence() {
    let sealed_elsewhere = Keypair::from_seed([1u8; 32])
        .unwrap()
        .sign(
            pairing().presence_salt(),
            1,
            Presence { endpoints: vec![] }.seal(&[8u8; 32]).unwrap(),
        )
        .unwrap();
    for (held, expect) in [
        (None, "no presence is published under this pairing"),
        (
            Some(sealed_elsewhere),
            "will not open under this pairing salt",
        ),
        (
            Some(presence(dead())),
            "nothing answered the punch inside its window",
        ),
    ] {
        let b = Bench::new(dead(), held);
        let channel = b.channel().tuned(
            Roving {
                window: Duration::from_millis(300),
                ..b.roving()
            },
            b.clock.arc(),
        );
        let said = channel.ask(&request(1)).unwrap_err().said();
        assert!(said.ends_with(expect), "{said}");
    }
    let b = Bench::new(dead(), None);
    let channel = b.channel().tuned(
        Roving {
            bootstrap: vec![String::new()],
            ..b.roving()
        },
        b.clock.arc(),
    );
    let said = channel.ask(&request(1)).unwrap_err().said();
    assert!(said.ends_with("no bootstrap node resolved"), "{said}");
}

#[test]
fn the_defaults_are_the_stated_ones() {
    let r = Roving::default();
    assert_eq!(r.direct, Duration::from_secs(5));
    assert_eq!(r.window, Duration::from_secs(40));
    assert_eq!(r.bootstrap.len(), 2);
    assert_eq!(r.advertise, None);
    assert_eq!(r.dht.k, 8);
}
