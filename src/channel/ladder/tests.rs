//! Every rung taken, against a fake DHT on loopback UDP and a stand-in
//! engine on a held line. The bench every file under here stands on is
//! this one's; `held` is what happens to a line between asks, `refusals`
//! is every rung falling through, `call` is what the call carries and from
//! which port, `recall` is rung 3 and its cache, and `said` is every line the
//! climb says, off a captured sink.

mod call;
mod held;
mod recall;
mod refusals;
mod said;

use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::{Arc, mpsc};
use std::time::Duration;

use serde_json::{Value, json};

use super::*;
use crate::channel::line::{GONE, PING};
use crate::channel::material::{ADDRESS, Whose, read_dir};
use crate::channel::rendezvous::item::{Call, Presence};
use crate::channel::rendezvous::tests::{pairing, provision};
use crate::channel::say::Say;
use crate::channel::{Channel, Reach};
use crate::dht::tests::fake::{FakeNode, Mood};
use crate::dht::tests::quick;
use crate::dht::{Keypair, Mutable, NodeId};
use crate::test_support::clock::FakeClock;
use crate::test_support::roving::{Fate, Reply, Roving as Engine};
use crate::test_support::{Scratch, dead, mint};

/// How long the bench lets a dial, a DHT query or a punch run before it is
/// HUNG — never how long a slow box gets (bl-73f2). Each ends the moment its
/// answer lands, so a healthy box spends milliseconds; only what nothing will
/// answer waits it out, and every test that leaves something unanswered
/// tunes its own short bound, which is then a floor and never a bet. A 2 s
/// walk deadline and a 3 s punch window were bets, and a loaded box lost
/// them. It sits under the coverage runner's own 300 s, so the failure that
/// names the wait is this suite's and not the runner's.
const HUNG: Duration = Duration::from_mins(4);

/// A scratch box: the operator's material, the fixture pairing, an address,
/// and one fake DHT node answering as the commons behind a bootstrap router
/// that names it — the router is a door and never holds an item.
struct Bench {
    scratch: Scratch,
    node: FakeNode,
    door: FakeNode,
    clock: FakeClock,
    /// Every line the rungs said, off the captured sink.
    heard: mpsc::Receiver<String>,
    say: Say,
}

impl Bench {
    /// `address` is what the entry names; `held` is what presence publishes,
    /// if anything.
    fn new(address: SocketAddr, held: Option<Mutable>) -> Bench {
        Bench::seen(address, held, Mood::Answer)
    }

    /// As [`Bench::new`], over a node of `mood` — how it says it sees us.
    fn seen(address: SocketAddr, held: Option<Mutable>, mood: Mood) -> Bench {
        let scratch = Scratch::new();
        mint::material(scratch.path());
        provision(scratch.path());
        std::fs::write(scratch.join(ADDRESS), address.to_string()).unwrap();
        let mut node = FakeNode::bind(NodeId([1u8; 20]));
        node.serve(vec![], mood, held.into_iter().collect());
        let mut door = FakeNode::bind(NodeId([0u8; 20]));
        door.serve(vec![node.node()], Mood::Router, vec![]);
        let (tx, heard) = mpsc::channel();
        Bench {
            scratch,
            node,
            door,
            clock: FakeClock::new(),
            heard,
            say: Say::new(Arc::new(move |line: &str| {
                let _ = tx.send(line.to_owned());
            })),
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
            direct: HUNG,
            window: HUNG,
            dht: Config {
                deadline: HUNG,
                ..quick()
            },
            bootstrap: vec![self.door.addr.to_string()],
            advertise: Some(vec![IpAddr::V4(Ipv4Addr::LOCALHOST)]),
            say: self.say.clone(),
        }
    }

    /// What the rungs said since this was last asked.
    fn said(&self) -> Vec<String> {
        self.heard.try_iter().collect()
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

/// Wait until this end's held line can see it is gone, judged at the bench's
/// clock — so a test that drops it by advancing time advances first. Waited
/// for, not slept past: a fixed pause is a bet against the box's load, and a
/// loaded builder collected (bl-7b83). Sleep first, so the sleep is run.
fn ended(b: &Bench) {
    let key = b.scratch.path().display().to_string();
    let now = b.clock.arc().now();
    till(&|| crate::state::worked(&key, |w| w.held.iter_mut().any(|h| h.gone(now).is_some())));
}

/// Wait until `done` says so — the one way this suite waits on another
/// thread. Sleep first, so the sleep is run. Not generic: under the llvm
/// coverage engine a generic copy per caller left two of its lines counted
/// uncovered although every caller ran them.
fn till(done: &dyn Fn() -> bool) {
    loop {
        std::thread::sleep(Duration::from_millis(20));
        if done() {
            break;
        }
    }
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
