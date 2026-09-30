//! **Rungs 3 and 4: the commons** (yog's `docs/REMOTE.md` §13.3–§13.4, ruling
//! bl-278f; DESIGN §4.40) — the two rungs that touch the DHT, split from the
//! ladder at the design-time budget, and the ones whose every step is said
//! ([`say`]).
//!
//! 3. **The re-call** — with the presence the last landed call answered to
//!    still cached, write a fresh call and punch: one walk, not two.
//! 4. **The full rendezvous** — read presence, then the same call and punch.
//!
//! There is no client-only re-punch: an engine behind a NAT holds no mapping
//! once its served stream ends and sends no SYN outside a call's window, so a
//! dropped line is always a call. The cache holds only what a call that
//! LANDED answered to, and a call that expires unanswered clears it, so the
//! next rung — or the next climb — reads presence again.

use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::Arc;

use crate::channel::rendezvous::Pairing;
use crate::channel::rendezvous::item::{Call, Presence};
use crate::channel::rendezvous::punch::{self, Punch};
use crate::channel::{Channel, say};
use crate::dht::{Dht, Udp};
use crate::state;

/// How a written call is said: [`say::call`] or [`say::recall`].
type Written = fn(u64, i64, &[SocketAddr], usize) -> String;

/// The re-call where presence is cached, else presence off the commons —
/// either way a call into the inbox and a punch.
pub(super) fn rungs(
    ch: &Channel,
    pairing: &Pairing,
    said: &mut Vec<String>,
) -> Result<TcpStream, String> {
    let bootstrap: Vec<SocketAddr> = ch
        .roving
        .bootstrap
        .iter()
        .filter_map(|name| name.to_socket_addrs().ok())
        .flatten()
        .collect();
    if bootstrap.is_empty() {
        said.push(say::no_bootstrap());
        return Err("no bootstrap node resolved".to_owned());
    }
    let udp = Udp::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0))
        .map_err(|e| format!("udp: {e}"))?;
    let mut dht = Dht::new(Box::new(udp), bootstrap, ch.roving.dht.clone())?;
    let cached = state::worked(&ch.key, |w| w.endpoints.clone());
    if !cached.is_empty()
        && let Some(tcp) = call(ch, pairing, &mut dht, cached, say::recall, said)?
    {
        return Ok(tcp);
    }
    let got = dht.get(pairing.engine, pairing.presence_salt());
    let Some(item) = got.map_err(|e| withheld(said, "presence not read", e))? else {
        said.push(say::no_presence());
        return Err("no presence is published under this pairing".to_owned());
    };
    let Some(presence) = Presence::open(&pairing.seal_key(), &item.value) else {
        said.push(say::unopened(item.seq));
        return Err("the presence item will not open under this pairing salt".to_owned());
    };
    said.push(say::presence(item.seq, &presence.endpoints));
    call(ch, pairing, &mut dht, presence.endpoints, say::call, said)?
        .ok_or_else(|| "nothing answered the punch inside its window".to_owned())
}

/// Write a call naming this box and punch toward `targets`, the engine's
/// endpoints at its punch port: the stream, `None` when the window passed
/// unanswered (and the cache with it), or why the call was never written.
fn call(
    ch: &Channel,
    pairing: &Pairing,
    dht: &mut Dht,
    targets: Vec<SocketAddr>,
    written: Written,
    said: &mut Vec<String>,
) -> Result<Option<TcpStream>, String> {
    let punch = punch_for(ch)?;
    // The route-local addresses, then every address a walk's nodes agreed
    // they saw us at that is not one of them (yog bl-efae) — all at the
    // punch port. The observed PORT is the DHT socket's UDP mapping, not the
    // punch port's, so only the address is taken (yog's `docs/REMOTE.md`
    // §13.2; a carrier that rewrites the port, §13.8, is the case this does
    // not reach). A re-call has walked nowhere yet, so it carries what the
    // last walk saw.
    let mut ips = ch.roving.advertise.clone().unwrap_or_else(punch::local_ips);
    for ip in seen(ch, dht) {
        if !ips.contains(&ip) {
            ips.push(ip);
        }
    }
    said.extend(say::overlay(&ips));
    let endpoints: Vec<SocketAddr> = ips
        .into_iter()
        .map(|ip| SocketAddr::new(ip, punch.port()))
        .collect();
    let mut nonce = [0u8; 8];
    crate::dht::random(&mut nonce)?;
    let call = Call {
        nonce: u64::from_be_bytes(nonce),
        endpoints: endpoints.clone(),
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
    let unwritten = format!("call nonce {} not written", call.nonce);
    let acks = dht.put(signed).map_err(|e| withheld(said, &unwritten, e))?;
    said.push(written(call.nonce, seq, &endpoints, acks));
    let what = format!("punch for nonce {}", call.nonce);
    said.push(say::started(&what, &targets, ch.roving.window));
    let tcp = punch.punch(targets.clone(), ch.roving.window);
    said.push(match &tcp {
        Some(tcp) => say::landed(&what, tcp.peer_addr().ok().map(|at| at.ip())),
        None => say::expired(&what, ch.roving.window),
    });
    state::worked(&ch.key, |w| {
        if tcp.is_some() {
            w.endpoints = targets;
        } else {
            w.endpoints.clear();
            w.observed.clear();
        }
    });
    Ok(tcp)
}

/// Where this end was last seen from the commons: the fresh walk's votes
/// when it cast any, kept for a re-call that has not walked yet.
fn seen(ch: &Channel, dht: &Dht) -> Vec<IpAddr> {
    let fresh: Vec<IpAddr> = dht.observed().iter().map(SocketAddr::ip).collect();
    state::worked(&ch.key, |w| {
        if !fresh.is_empty() {
            w.observed = fresh;
        }
        w.observed.clone()
    })
}

/// A walk that failed: said without its reason, handed back with it.
fn withheld(said: &mut Vec<String>, what: &str, refusal: String) -> String {
    said.push(say::withheld(what));
    refusal
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
