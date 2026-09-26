//! Every rung taken, against a fake DHT on loopback UDP and a stand-in
//! engine on a held line. The bench every file under here stands on is
//! this one's; `held` is what happens to a line between asks, `refusals`
//! is every rung falling through.

mod held;
mod refusals;

use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::time::Duration;

use serde_json::{Value, json};

use super::*;
use crate::channel::line::{GONE, PING};
use crate::channel::material::{ADDRESS, Whose, read_dir};
use crate::channel::rendezvous::item::Call;
use crate::channel::rendezvous::tests::{pairing, provision};
use crate::channel::{Channel, Reach};
use crate::dht::tests::fake::{FakeNode, Mood};
use crate::dht::tests::quick;
use crate::dht::{Keypair, Mutable, NodeId};
use crate::test_support::clock::FakeClock;
use crate::test_support::roving::{Fate, Reply, Roving as Engine};
use crate::test_support::{Scratch, mint};

/// A scratch box: the operator's material, the fixture pairing, an address,
/// and one fake DHT node answering as the commons behind a bootstrap router
/// that names it — the router is a door and never holds an item.
struct Bench {
    scratch: Scratch,
    node: FakeNode,
    door: FakeNode,
    clock: FakeClock,
}

impl Bench {
    /// `address` is what the entry names; `held` is what presence publishes,
    /// if anything.
    fn new(address: SocketAddr, held: Option<Mutable>) -> Bench {
        let scratch = Scratch::new();
        mint::material(scratch.path());
        provision(scratch.path());
        std::fs::write(scratch.join(ADDRESS), address.to_string()).unwrap();
        let mut node = FakeNode::bind(NodeId([1u8; 20]));
        node.serve(vec![], Mood::Answer, held.into_iter().collect());
        let mut door = FakeNode::bind(NodeId([0u8; 20]));
        door.serve(vec![node.node()], Mood::Router, vec![]);
        Bench {
            scratch,
            node,
            door,
            clock: FakeClock::new(),
        }
    }

    fn channel(&self) -> Channel {
        let material = read_dir(self.scratch.path(), Whose::Own).unwrap().unwrap();
        assert_eq!(material.pairing, Some(pairing()));
        Channel::open(&material)
            .unwrap()
            .tuned(self.roving(), self.clock.arc())
    }

    fn roving(&self) -> Roving {
        Roving {
            direct: Duration::from_millis(300),
            window: Duration::from_secs(3),
            dht: quick(),
            bootstrap: vec![self.door.addr.to_string()],
            advertise: Some(vec![IpAddr::V4(Ipv4Addr::LOCALHOST)]),
        }
    }

    /// The call the seat wrote into the inbox, opened.
    fn call(&self) -> Option<Call> {
        let inbox = pairing().inbox_keypair().unwrap().public();
        self.node
            .items()
            .iter()
            .find(|item| item.key == inbox)
            .and_then(|item| Call::open(&pairing().seal_key(), &item.value))
    }

    /// The punch port this entry was given for the run.
    fn punch_port(&self) -> u16 {
        crate::state::worked(&self.scratch.path().display().to_string(), |w| {
            w.punch.as_ref().unwrap().port()
        })
    }
}

/// Presence under the fixture engine's own seed, naming `at`.
fn presence(at: SocketAddr) -> Mutable {
    let p = pairing();
    let sealed = Presence {
        endpoints: vec![at],
    }
    .seal(&p.seal_key())
    .unwrap();
    Keypair::from_seed([1u8; 32])
        .unwrap()
        .sign(p.presence_salt(), 1, sealed)
        .unwrap()
}

/// A loopback port nobody listens on.
fn dead() -> SocketAddr {
    let gone = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = gone.local_addr().unwrap();
    drop(gone);
    addr
}

/// A listener, and its address.
fn listener() -> (TcpListener, SocketAddr) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    (listener, addr)
}

fn request(n: u64) -> Value {
    json!({"op": "workspaces", "n": n})
}

fn ops(heard: &[Value]) -> Vec<u64> {
    heard.iter().filter_map(|v| v.get("n")?.as_u64()).collect()
}

#[test]
fn rung_two_the_direct_address_answers_and_is_not_held() {
    let (listener, at) = listener();
    let b = Bench::new(at, None);
    let engine = Engine::listen(b.scratch.path(), listener, vec![Reply::yes(), Reply::yes()]);
    let channel = b.channel();
    assert!(channel.ask(&request(1)).is_ok());
    assert!(channel.ask(&request(2)).is_ok());
    assert_eq!(engine.connections(), 2, "a direct line is one per ask");
    assert_eq!(b.node.queries(), 0, "the commons was never asked");
    assert_eq!(b.call(), None);
}

#[test]
fn rung_four_reads_presence_writes_a_call_punches_and_the_line_is_held() {
    let (listener, at) = listener();
    let b = Bench::new(dead(), Some(presence(at)));
    let engine = Engine::listen(b.scratch.path(), listener, vec![Reply::yes(), Reply::yes()]);
    let channel = b.channel();
    assert_eq!(
        channel.ask(&request(1)),
        Ok(vec![Reply::yes().frames[0].clone()])
    );
    let asked = b.node.queries();
    assert!(asked > 0, "the commons was read and written");
    assert_eq!(
        b.call().map(|c| c.endpoints),
        Some(vec![SocketAddr::new(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            b.punch_port()
        )]),
        "the call names this box at its punch port"
    );
    assert!(channel.ask(&request(2)).is_ok());
    assert_eq!(engine.connections(), 1, "the second ask rode the held line");
    assert_eq!(
        b.node.queries(),
        asked,
        "and touched the commons not at all"
    );
    let heard = engine.heard();
    assert_eq!(ops(&heard), vec![1, 2]);
    assert_eq!(
        heard.iter().filter(|v| v.get("protocol").is_some()).count(),
        1,
        "one preface per connection, none per ask"
    );
}

#[test]
fn rung_three_punches_again_where_the_engine_was_without_the_commons() {
    let (listener, at) = listener();
    let b = Bench::new(dead(), Some(presence(at)));
    let script = vec![Reply::yes().then(Fate::Fin), Reply::yes()];
    let engine = Engine::listen(b.scratch.path(), listener, script);
    let channel = b.channel();
    assert!(channel.ask(&request(1)).is_ok());
    let asked = b.node.queries();
    assert!(channel.ask(&request(2)).is_ok());
    assert_eq!(engine.connections(), 2, "the closed line was not reused");
    assert_eq!(
        b.node.queries(),
        asked,
        "the re-punch cost no DHT round trip"
    );
}

#[test]
fn the_engines_own_syn_lands_on_this_listener() {
    let b = Bench::new(dead(), Some(presence(dead())));
    let engine = Engine::call_back(
        b.scratch.path(),
        b.node.store(),
        pairing(),
        vec![Reply::yes()],
    );
    let channel = b.channel();
    assert!(channel.ask(&request(1)).is_ok());
    assert_eq!(engine.connections(), 1);
    assert_eq!(ops(&engine.heard()), vec![1]);
}

#[test]
fn the_punch_port_is_bound_once_for_the_run() {
    let (listener, at) = listener();
    let b = Bench::new(dead(), Some(presence(at)));
    let script = vec![Reply::yes().then(Fate::Fin), Reply::yes()];
    let engine = Engine::listen(b.scratch.path(), listener, script);
    let channel = b.channel();
    assert!(channel.ask(&request(1)).is_ok());
    let (port, asked) = (b.punch_port(), b.node.queries());
    // As if the engine had never been found this run — but the port stands.
    crate::state::worked(&b.scratch.path().display().to_string(), |w| {
        w.endpoints.clear();
    });
    assert!(channel.ask(&request(2)).is_ok());
    assert_eq!(engine.connections(), 2);
    assert!(b.node.queries() > asked, "the commons was asked again");
    assert_eq!(b.punch_port(), port, "the same port, so the peer sees one");
    assert_eq!(
        b.call().map(|c| c.endpoints),
        Some(vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)])
    );
}
