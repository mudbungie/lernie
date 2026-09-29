//! **Rung 4: the full rendezvous** (yog's `docs/REMOTE.md` §13.4; DESIGN
//! §4.40) — read presence off the commons, write a sealed call into the
//! inbox, punch. The one rung that touches the DHT, split from the ladder at
//! the design-time budget, and the one whose every step is said ([`say`]).

use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::Arc;

use super::punched;
use crate::channel::rendezvous::Pairing;
use crate::channel::rendezvous::item::{Call, Presence};
use crate::channel::rendezvous::punch::{self, Punch};
use crate::channel::{Channel, say};
use crate::dht::{Dht, Udp};
use crate::state;

/// Presence off the commons, a call into the inbox, a punch.
pub(super) fn rung(
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
    let written = format!("call nonce {} not written", call.nonce);
    let acks = dht.put(signed).map_err(|e| withheld(said, &written, e))?;
    said.push(say::call(call.nonce, seq, &endpoints, acks));
    let targets = presence.endpoints;
    state::worked(&ch.key, |w| w.endpoints.clone_from(&targets));
    let what = format!("punch for nonce {}", call.nonce);
    punched(&punch, &what, targets, ch.roving.window, said)
        .ok_or_else(|| "nothing answered the punch inside its window".to_owned())
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
