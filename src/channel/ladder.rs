//! **The four-rung dial ladder** (yog's `docs/REMOTE.md` §13.4; DESIGN
//! §4.40): what a dial climbs to get a connection, cheapest first.
//!
//! 1. **A live held line** — a punched connection kept from an earlier ask,
//!    still inside the silence bound and still open ([`Held::gone`]).
//! 2. **The entry's direct address** — a LAN, a stable server, loopback. An
//!    entry with no rendezvous material stops here, exactly as it always did:
//!    the connect is unbounded and its refusal is the sentence. With material
//!    below it the connect is bounded, because a NAT that drops the SYN would
//!    otherwise cost minutes before the next rung is tried.
//! 3. **A re-punch at the RAM-cached endpoints** — where the engine was last
//!    found, without a DHT round trip.
//! 4. **The full rendezvous** — read presence off the commons, write a sealed
//!    call into the inbox, punch. The DHT is touched here and nowhere else:
//!    a seat pays nothing for the machinery when idle.
//!
//! What worked stays RAM for the run and never disk ([`crate::state`]): the
//! endpoints, the punch port, the held lines. Whichever rung answered, the
//! stream is handed back as a bare `TcpStream` and the caller runs the same
//! inner mTLS over it, verifying the same engine name off the same address —
//! the ladder decides how a socket is obtained and nothing about what is
//! trusted on it.
//!
//! **Every rung is said** ([`say`]; yog's REMOTE §13.4, "The operator's view
//! of the loop"): on an entry with rendezvous material, each climb's rungs,
//! punches and held-line events reach stderr as families and counts, never
//! an address, and a climb that saw what the last one saw is not said again.

use std::net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

use super::line::{Held, Line};
use super::rendezvous::punch::Punch;
use super::{Channel, say};
use crate::dht::Config;
use crate::state;

/// Rung 4, the one that touches the commons.
mod commons;

/// Every knob the roving rungs turn — stated defaults, and a test's to
/// shorten.
#[derive(Clone, Debug)]
pub struct Roving {
    /// How long the direct rung waits for a SYN to be answered when a rung
    /// stands below it.
    pub direct: Duration,
    /// How long a punch keeps sending SYNs: the engine's poll period plus
    /// its own punch window, so a call written at the worst moment is still
    /// answered inside it.
    pub window: Duration,
    /// The DHT walk's parameters.
    pub dht: Config,
    /// The mainline's bootstrap nodes, resolved at the moment of a rendezvous
    /// and never before — a name lookup is a network act. The four yog's
    /// `mainline()` names: each is a door into the keyspace and never a
    /// result, and the walk re-asks them whenever its frontier runs dry.
    pub bootstrap: Vec<String>,
    /// The addresses this box advertises in a call, at the punch port; `None`
    /// is the box's own route-local addresses, read when the call is written.
    pub advertise: Option<Vec<IpAddr>>,
    /// Where the rungs' lines go ([`say`]): stderr unless replaced.
    pub say: say::Say,
}

impl Default for Roving {
    fn default() -> Roving {
        Roving {
            direct: Duration::from_secs(5),
            window: Duration::from_secs(40),
            dht: Config::default(),
            bootstrap: vec![
                "router.bittorrent.com:6881".to_owned(),
                "dht.transmissionbt.com:6881".to_owned(),
                "router.utorrent.com:6881".to_owned(),
                "dht.aelitis.com:6881".to_owned(),
            ],
            advertise: None,
            say: say::Say::default(),
        }
    }
}

/// Climb the ladder for `ch`: a line with a stream on it, or the sentence
/// saying why none of the rungs answered. `held` says whether the first rung
/// is climbed at all. What the climb saw is said once it is over
/// ([`say::tell`]); an entry with no rendezvous material says nothing.
pub(crate) fn climb(ch: &Channel, held: bool) -> Result<Line, String> {
    let mut said = Vec::new();
    let line = rungs(ch, held, &mut said);
    say::tell(ch, "climb", said);
    line
}

fn rungs(ch: &Channel, held: bool, said: &mut Vec<String>) -> Result<Line, String> {
    if held && let Some(line) = rung_held(ch, said) {
        return Ok(line);
    }
    let direct = rung_direct(ch);
    if ch.pairing.is_some() {
        let outcome = direct.as_ref().map(|_| ()).map_err(std::io::Error::kind);
        said.push(say::direct(outcome, ch.roving.direct));
    }
    let direct = match direct {
        Ok(tcp) => return ch.line(tcp, false),
        Err(e) => format!("connect {}: {e}", ch.address),
    };
    let Some(pairing) = ch.pairing else {
        return Err(direct);
    };
    let tcp = match rung_repunch(ch, said) {
        Some(tcp) => tcp,
        None => commons::rung(ch, &pairing, said)
            .map_err(|refusal| format!("{direct}; rendezvous: {refusal}"))?,
    };
    said.push(say::kept());
    ch.line(tcp, true)
}

/// Rung 1: the newest held line that is still alive. The pool is taken out
/// of the lock to be asked — a liveness check touches the socket — and what
/// was not taken goes back. A line found gone is dropped, and said with why.
fn rung_held(ch: &Channel, said: &mut Vec<String>) -> Option<Line> {
    let mut pool = state::worked(&ch.key, |w| std::mem::take(&mut w.held));
    let now = ch.clock.now();
    let mut chosen: Option<Held> = None;
    while chosen.is_none()
        && let Some(mut held) = pool.pop()
    {
        match held.gone(now) {
            None => chosen = Some(held),
            Some(why) => said.push(say::dropped(why)),
        }
    }
    state::worked(&ch.key, |w| w.held.append(&mut pool));
    let line = chosen.map(Line::from_held);
    if line.is_some() {
        said.push(say::taken());
    }
    line
}

/// Rung 2: the address the entry names.
fn rung_direct(ch: &Channel) -> std::io::Result<TcpStream> {
    if ch.pairing.is_none() {
        TcpStream::connect(&ch.address)
    } else {
        bounded(&ch.address, ch.roving.direct)
    }
}

/// A connect that gives each address of `address` at most `within`.
fn bounded(address: &str, within: Duration) -> std::io::Result<TcpStream> {
    let mut refusal = std::io::Error::other("resolved to no address");
    for addr in address.to_socket_addrs()? {
        match TcpStream::connect_timeout(&addr, within) {
            Ok(tcp) => return Ok(tcp),
            Err(e) => refusal = e,
        }
    }
    Err(refusal)
}

/// Rung 3: punch again at the endpoints the last rendezvous found.
fn rung_repunch(ch: &Channel, said: &mut Vec<String>) -> Option<TcpStream> {
    let (endpoints, punch) = state::worked(&ch.key, |w| (w.endpoints.clone(), w.punch.clone()));
    if endpoints.is_empty() {
        return None;
    }
    let punch = punch?;
    punched(&punch, "re-punch", endpoints, ch.roving.window, said)
}

/// A punch, said as it starts and as it ends: `what` names which.
fn punched(
    punch: &Punch,
    what: &str,
    targets: Vec<SocketAddr>,
    window: Duration,
    said: &mut Vec<String>,
) -> Option<TcpStream> {
    said.push(say::started(what, &targets, window));
    let tcp = punch.punch(targets, window);
    said.push(match &tcp {
        Some(tcp) => say::landed(what, tcp.peer_addr().ok().map(|at| at.ip())),
        None => say::expired(what, window),
    });
    tcp
}

#[cfg(test)]
mod tests;
