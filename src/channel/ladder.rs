//! **The four-rung dial ladder** (yog's `docs/REMOTE.md` §13.4; DESIGN
//! §4.40): what a dial climbs to get a connection, cheapest first.
//!
//! 1. **A live held line** — a punched connection kept from an earlier ask,
//!    still inside the silence bound and still open ([`Held::alive`]).
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

use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;

use super::Channel;
use super::line::{Held, Line};
use super::rendezvous::Pairing;
use super::rendezvous::item::{Call, Presence};
use super::rendezvous::punch::{self, Punch};
use crate::dht::{Config, Dht, Udp};
use crate::state;

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
        }
    }
}

/// Climb the ladder for `ch`: a line with a stream on it, or the sentence
/// saying why none of the rungs answered. `held` says whether the first rung
/// is climbed at all.
pub(crate) fn climb(ch: &Channel, held: bool) -> Result<Line, String> {
    if held && let Some(line) = rung_held(ch) {
        return Ok(line);
    }
    let direct = match rung_direct(ch) {
        Ok(tcp) => return ch.line(tcp, false),
        Err(refusal) => refusal,
    };
    let Some(pairing) = ch.pairing else {
        return Err(direct);
    };
    if let Some(tcp) = rung_repunch(ch) {
        return ch.line(tcp, true);
    }
    match rung_rendezvous(ch, &pairing) {
        Ok(tcp) => ch.line(tcp, true),
        Err(refusal) => Err(format!("{direct}; rendezvous: {refusal}")),
    }
}

/// Rung 1: the newest held line that is still alive. The pool is taken out
/// of the lock to be asked — a liveness check touches the socket — and what
/// was not taken goes back.
fn rung_held(ch: &Channel) -> Option<Line> {
    let mut pool = state::worked(&ch.key, |w| std::mem::take(&mut w.held));
    let now = ch.clock.now();
    let mut chosen: Option<Held> = None;
    while chosen.is_none()
        && let Some(mut held) = pool.pop()
    {
        if held.alive(now) {
            chosen = Some(held);
        }
    }
    state::worked(&ch.key, |w| w.held.append(&mut pool));
    chosen.map(Line::from_held)
}

/// Rung 2: the address the entry names.
fn rung_direct(ch: &Channel) -> Result<TcpStream, String> {
    let tcp = if ch.pairing.is_none() {
        TcpStream::connect(&ch.address)
    } else {
        bounded(&ch.address, ch.roving.direct)
    };
    tcp.map_err(|e| format!("connect {}: {e}", ch.address))
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
fn rung_repunch(ch: &Channel) -> Option<TcpStream> {
    let (endpoints, punch) = state::worked(&ch.key, |w| (w.endpoints.clone(), w.punch.clone()));
    if endpoints.is_empty() {
        return None;
    }
    punch?.punch(endpoints, ch.roving.window)
}

/// Rung 4: presence off the commons, a call into the inbox, a punch.
fn rung_rendezvous(ch: &Channel, pairing: &Pairing) -> Result<TcpStream, String> {
    let bootstrap: Vec<SocketAddr> = ch
        .roving
        .bootstrap
        .iter()
        .filter_map(|name| name.to_socket_addrs().ok())
        .flatten()
        .collect();
    if bootstrap.is_empty() {
        return Err("no bootstrap node resolved".to_owned());
    }
    let udp = Udp::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0))
        .map_err(|e| format!("udp: {e}"))?;
    let mut dht = Dht::new(Box::new(udp), bootstrap, ch.roving.dht.clone())?;
    let Some(item) = dht.get(pairing.engine, pairing.presence_salt())? else {
        return Err("no presence is published under this pairing".to_owned());
    };
    let Some(presence) = Presence::open(&pairing.seal_key(), &item.value) else {
        return Err("the presence item will not open under this pairing salt".to_owned());
    };
    let punch = punch_for(ch)?;
    // The route-local addresses, then every address the presence walk's
    // nodes agreed they saw us at that is not one of them (yog bl-efae) —
    // all at the punch port. The observed PORT is the DHT socket's UDP
    // mapping, not the punch port's, so only the address is taken (yog's
    // `docs/REMOTE.md` §13.2; a carrier that rewrites the port, §13.8, is
    // the case this does not reach).
    let mut ips = ch.roving.advertise.clone().unwrap_or_else(punch::local_ips);
    for ip in dht.observed().iter().map(SocketAddr::ip) {
        if !ips.contains(&ip) {
            ips.push(ip);
        }
    }
    let endpoints = ips
        .into_iter()
        .map(|ip| SocketAddr::new(ip, punch.port()))
        .collect();
    let mut nonce = [0u8; 8];
    crate::dht::random(&mut nonce)?;
    let call = Call {
        nonce: u64::from_be_bytes(nonce),
        endpoints,
    };
    let sealed = call.seal(&pairing.seal_key())?;
    let unix = ch.clock.unix();
    let seq = state::worked(&ch.key, |w| {
        w.last_seq = unix.max(w.last_seq + 1);
        w.last_seq
    });
    let signed = pairing
        .inbox_keypair()?
        .sign(pairing.inbox_salt(), seq, sealed)?;
    dht.put(signed)?;
    let targets = presence.endpoints;
    state::worked(&ch.key, |w| w.endpoints.clone_from(&targets));
    match punch.punch(targets, ch.roving.window) {
        Some(tcp) => Ok(tcp),
        None => Err("nothing answered the punch inside its window".to_owned()),
    }
}

/// The entry's punch port for the run: bound once, kept in RAM.
fn punch_for(ch: &Channel) -> Result<Arc<Punch>, String> {
    if let Some(punch) = state::worked(&ch.key, |w| w.punch.clone()) {
        return Ok(punch);
    }
    let bound = Arc::new(Punch::bind(0)?);
    Ok(state::worked(&ch.key, |w| {
        Arc::clone(w.punch.get_or_insert(bound))
    }))
}

#[cfg(test)]
mod tests;
