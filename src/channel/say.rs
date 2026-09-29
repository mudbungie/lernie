//! **What the rendezvous path says on the seat's stderr** (yog's
//! `docs/REMOTE.md` §13.4 "The operator's view of the loop", the engine's
//! bl-355c; DESIGN §4.40): one line per rung, punch and held-line event, in
//! the engine's `yog: rendezvous: …` shape under this binary's name. Before
//! these a live dial could be watched only from socket state on the engine.
//!
//! Every line is built here and nowhere else, so the one rule they share is
//! enforced by the only file that could break it: **counts, sequence numbers,
//! nonces and address families — never an address, a key, a salt or a sealed
//! byte.** A DHT failure is said without its reason for the engine's reason:
//! the walk's refusals name the nodes it asked. The sentence the dial hands
//! back to its caller is unchanged; this is the diagnosis beside it.
//!
//! **A repeated outcome is said once** ([`tell`]). What one climb of the
//! ladder saw is said together, and only when it differs from what the last
//! climb for the same entry saw — so a window asking every beat over a held
//! line says `held line taken up` once, not every beat, and a direct address
//! refusing every beat is said once until something changes. A rendezvous
//! carries a fresh nonce, so it is always said. A ping discarded while an
//! answer was read is the second transcript, judged the same way.
//!
//! The sink is injected ([`Say`], a knob of [`super::ladder::Roving`]) so the
//! suite reads the lines the ladder emits; the default is stderr.

use std::fmt;
use std::io::ErrorKind;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use super::Channel;

/// Where the rendezvous path's lines go.
#[derive(Clone)]
pub struct Say(Arc<dyn Fn(&str) + Send + Sync>);

impl Say {
    /// A sink of the embedder's own.
    pub fn new(sink: Arc<dyn Fn(&str) + Send + Sync>) -> Say {
        Say(sink)
    }

    fn line(&self, line: &str) {
        (self.0)(line);
    }
}

/// The seat's sink: the process's stderr, a line at a time — a diagnosis,
/// so never stdout, where `--json` puts its one envelope.
impl Default for Say {
    fn default() -> Say {
        Say(Arc::new(|line: &str| eprintln!("{line}")))
    }
}

impl fmt::Debug for Say {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Say")
    }
}

/// Say `lines` — what `source` saw for `ch`'s entry — unless it is exactly
/// what `source` saw last time. The memory is the entry's, shared by every
/// channel and thread that dials it ([`crate::state::worked`]).
pub(crate) fn tell(ch: &Channel, source: &'static str, lines: Vec<String>) {
    let news = crate::state::worked(&ch.key, |w| {
        w.said.insert(source, lines.clone()).as_ref() != Some(&lines)
    });
    if news {
        for line in &lines {
            ch.roving.say.line(line);
        }
    }
}

const P: &str = "lernie: rendezvous:";

pub(crate) fn taken() -> String {
    format!("{P} held line taken up — no dial")
}

pub(crate) fn dropped(why: &str) -> String {
    format!("{P} held line dropped — {why}")
}

/// The direct rung's outcome: answered, timed out inside its bound, or
/// refused in the transport's own class — never its message, which names
/// the address.
pub(crate) fn direct(outcome: Result<(), ErrorKind>, within: Duration) -> String {
    match outcome {
        Ok(()) => format!("{P} direct rung answered"),
        Err(ErrorKind::TimedOut) => format!("{P} direct rung timed out after {within:?}"),
        Err(kind) => format!("{P} direct rung refused — {kind}"),
    }
}

pub(crate) fn no_bootstrap() -> String {
    format!("{P} no bootstrap node resolved — no rendezvous")
}

/// A walk that failed, said without its reason: it names nodes.
pub(crate) fn withheld(what: &str) -> String {
    format!("{P} {what} — the DHT walk failed (reason withheld: it names nodes)")
}

pub(crate) fn no_presence() -> String {
    format!("{P} no presence is published under this pairing")
}

pub(crate) fn unopened(seq: i64) -> String {
    format!("{P} presence seq {seq} did not open under this pairing's seal key")
}

pub(crate) fn presence(seq: i64, endpoints: &[SocketAddr]) -> String {
    format!("{P} presence read — seq {seq}, {}", counted(endpoints))
}

/// Said before the window is spent when every address the call would carry
/// is in an overlay or carrier-NAT range: an engine outside that network has
/// nothing it can punch toward (yog bl-f612 decides the policy; this says
/// the fact).
pub(crate) fn overlay(ips: &[IpAddr]) -> Option<String> {
    (!ips.is_empty() && ips.iter().all(overlaid)).then(|| {
        format!(
            "{P} every address the call carries ({}) is in an overlay or carrier-NAT range — an engine outside that network cannot reach it",
            families(ips)
        )
    })
}

pub(crate) fn call(nonce: u64, seq: i64, endpoints: &[SocketAddr], acks: usize) -> String {
    format!(
        "{P} call nonce {nonce} written — seq {seq}, {}, {acks} ack(s)",
        counted(endpoints)
    )
}

pub(crate) fn started(what: &str, targets: &[SocketAddr], window: Duration) -> String {
    format!(
        "{P} {what} started — {}, window {window:?}",
        counted(targets)
    )
}

pub(crate) fn landed(what: &str, peer: Option<IpAddr>) -> String {
    format!("{P} {what} landed ({})", families(&Vec::from_iter(peer)))
}

pub(crate) fn expired(what: &str, window: Duration) -> String {
    format!("{P} {what} expired after {window:?} with no stream")
}

pub(crate) fn kept() -> String {
    format!("{P} punched line held between asks")
}

pub(crate) fn pinged() -> String {
    format!("{P} held line ping discarded")
}

/// `2 endpoint(s) (1 v6, 1 v4)`.
fn counted(endpoints: &[SocketAddr]) -> String {
    let ips: Vec<IpAddr> = endpoints.iter().map(SocketAddr::ip).collect();
    format!("{} endpoint(s) ({})", ips.len(), families(&ips))
}

/// How many of `ips` are of each family, v6 first — `1 v6, 2 v4`, or `none`.
fn families(ips: &[IpAddr]) -> String {
    let v6 = ips.iter().filter(|ip| ip.is_ipv6()).count();
    let parts: Vec<String> = [(v6, "v6"), (ips.len() - v6, "v4")]
        .into_iter()
        .filter(|(n, _)| *n > 0)
        .map(|(n, family)| format!("{n} {family}"))
        .collect();
    if parts.is_empty() {
        "none".to_owned()
    } else {
        parts.join(", ")
    }
}

/// RFC 6598's shared space (100.64/10, where carrier NAT and the common
/// overlays both live) or an IPv6 unique-local address (fc00::/7).
fn overlaid(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let [a, b, ..] = v4.octets();
            a == 100 && b & 0xc0 == 0x40
        }
        IpAddr::V6(v6) => v6.segments()[0] & 0xfe00 == 0xfc00,
    }
}

#[cfg(test)]
mod tests;
