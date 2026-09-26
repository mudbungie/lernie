//! The iterative walk, seen through the one verb that reads: it hops, it is
//! bounded, and every way a node misbehaves leaves it standing.

use super::*;

/// The item the far node holds — reachable only by hopping through `B`.
fn far() -> Mutable {
    keypair().sign(b"s".to_vec(), 1, b"far".to_vec()).unwrap()
}

#[test]
fn a_walk_hops_toward_the_target_and_finds_what_the_far_node_holds() {
    let mut nodes = topology();
    serve(&mut nodes, vec![], vec![far()]);
    let mut dht = client(vec![nodes[0].addr], quick());
    assert_eq!(
        dht.get(keypair().public(), b"s".to_vec()).unwrap(),
        Some(far())
    );
}

#[test]
fn a_walk_stops_at_its_query_cap() {
    let mut nodes = topology();
    serve(&mut nodes, vec![], vec![far()]);
    let mut dht = client(
        vec![nodes[0].addr],
        Config {
            max_queries: 1,
            ..quick()
        },
    );
    assert_eq!(dht.get(keypair().public(), b"s".to_vec()).unwrap(), None);
}

#[test]
fn no_bootstrap_is_an_error_before_any_datagram() {
    let mut dht = client(vec![], quick());
    assert_eq!(
        dht.get(keypair().public(), vec![]).unwrap_err(),
        "no bootstrap node to ask"
    );
}

#[test]
fn a_silent_commons_is_an_error() {
    let mut a = FakeNode::bind(id(0));
    a.serve(vec![], Mood::Silent, vec![]);
    let mut dht = client(vec![a.addr], quick());
    let key = keypair().public();
    let e = dht.get(key, vec![]).unwrap_err();
    assert_eq!(
        e,
        format!("no DHT node answered get for {}", target_of(&key, &[]))
    );
}

#[test]
fn a_zero_round_never_waits() {
    let mut a = FakeNode::bind(id(0));
    a.serve(vec![], Mood::Answer, vec![]);
    let mut dht = client(
        vec![a.addr],
        Config {
            round: Duration::ZERO,
            ..quick()
        },
    );
    assert!(
        dht.get(keypair().public(), vec![])
            .unwrap_err()
            .starts_with("no DHT node answered")
    );
}

#[test]
fn a_node_that_refuses_is_heard_but_is_no_result() {
    let mut a = FakeNode::bind(id(0));
    a.serve(vec![], Mood::Refuse, vec![]);
    let mut dht = client(vec![a.addr], quick());
    assert_eq!(dht.get(keypair().public(), vec![]).unwrap(), None);
}

#[test]
fn noise_on_the_socket_is_not_an_answer() {
    let mut garbage = FakeNode::bind(id(1));
    garbage.serve(vec![], Mood::Garbage, vec![]);
    let mut anonymous = FakeNode::bind(id(2));
    anonymous.serve(vec![], Mood::Anonymous, vec![]);
    let mut stray = FakeNode::bind(id(3));
    stray.serve(vec![], Mood::Stray, vec![]);
    let mut a = FakeNode::bind(id(4));
    a.serve(vec![], Mood::Answer, vec![far()]);
    let bootstrap = vec![garbage.addr, anonymous.addr, stray.addr, a.addr];
    let mut dht = client(
        bootstrap,
        Config {
            alpha: 4,
            ..quick()
        },
    );
    assert_eq!(
        dht.get(keypair().public(), b"s".to_vec()).unwrap(),
        Some(far())
    );
}
