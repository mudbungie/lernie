//! **A walk whose silence the fakes declare** (bl-7b83): loopback UDP to the
//! fake nodes, and a deadline that passes only when nothing is owed.
//!
//! On a real socket, *silent* and *slow* look the same until the deadline,
//! so a walk run against the system clock bets that every live fake answers
//! inside it — and a loaded box collected on that bet. Here each send asks
//! the fake it went to whether it will answer ([`ignores`], the rule the fake
//! itself serves by); every answer owed is waited for under a bound generous
//! enough to mean "hung", and only when nothing is owed does a read advance
//! the walk's clock by the whole wait it asked for. A silent node therefore
//! costs exactly one deadline of steered time and no wall time at all, and a
//! live node always beats it, however long it took.

use super::super::bencode::Value;
use super::super::transport::{Transport, Udp};
use super::fake::ignores;
use super::*;
use crate::test_support::clock::FakeClock;
use std::cell::Cell;

/// How long a read waits for a datagram it is owed before the walk is hung.
const HUNG: Duration = Duration::from_mins(1);

struct Steered {
    udp: Udp,
    moods: Vec<(SocketAddr, Mood)>,
    clock: FakeClock,
    owed: Cell<usize>,
}

impl Steered {
    /// Whether the fake at `to` lets this query go unanswered.
    fn ignored(&self, to: SocketAddr, bytes: &[u8]) -> bool {
        let verb = Value::decode(bytes).unwrap();
        let verb = verb.get("q").unwrap().as_bytes().unwrap();
        self.moods
            .iter()
            .any(|(at, mood)| *at == to && ignores(*mood, verb))
    }
}

impl Transport for Steered {
    fn send(&self, to: SocketAddr, bytes: &[u8]) -> io::Result<()> {
        self.udp.send(to, bytes)?;
        let owed = usize::from(!self.ignored(to, bytes));
        self.owed.set(self.owed.get() + owed);
        Ok(())
    }

    fn recv(&self, wait: Duration) -> io::Result<Option<(SocketAddr, Vec<u8>)>> {
        if self.owed.get() == 0 {
            self.clock.advance(wait);
            return Ok(None);
        }
        let got = self.udp.recv(HUNG)?.expect("an answering node answers");
        self.owed.set(self.owed.get() - 1);
        Ok(Some(got))
    }
}

/// A client over loopback that knows what each of `fakes` will answer, and
/// the clock its deadlines are read against — which moves only past
/// silence. A send to any other address is owed an answer.
pub(super) fn steered(
    bootstrap: Vec<SocketAddr>,
    config: Config,
    fakes: &[&FakeNode],
) -> (Dht, FakeClock) {
    let clock = FakeClock::new();
    let transport = Steered {
        udp: Udp::bind("127.0.0.1:0".parse().unwrap()).unwrap(),
        moods: fakes.iter().map(|n| (n.addr, n.mood.unwrap())).collect(),
        clock: clock.clone(),
        owed: Cell::new(0),
    };
    let mut dht = Dht::new(Box::new(transport), bootstrap, config).unwrap();
    dht.clock = clock.arc();
    (dht, clock)
}
