//! **The channel**: the seat's end of one wire to one engine (yog's
//! `docs/REMOTE.md` §3, §8, §8.2; DESIGN §4.2-4.4).
//!
//! **A seat dials and is never dialled.** Every leg is an answer to something
//! this end asked for, so there is no inbound direction to secure because there
//! is no inbound direction. That is not a property of this file — it is the
//! whole shape of it: there is [`Channel::ask`], there is [`Channel::follow`],
//! and there is nothing else.
//!
//! **One connection per ask** (REMOTE §3's cadence ruling: *"the seat polls"*,
//! at human cadence). A held connection is an optimisation of that same surface
//! rather than a different one — and where a read genuinely never finishes, it
//! is [`Channel::follow`]: the same one request, with each frame handed over as
//! it arrives instead of collected. No second envelope, no second reader.
//!
//! **And where the entry carries rendezvous material, the connection is
//! climbed for rather than dialled** (REMOTE §13.4; [`ladder`]): a live held
//! line, the direct address, a re-punch where the engine was last found, the
//! full rendezvous over the commons. A punched line is held between asks
//! ([`line`]), because first contact costs seconds. Nothing above `dial`
//! knows which rung answered, and nothing about what is trusted changes with
//! it — the same mTLS verifies the same name off the same address.
//!
//! **The engine's name comes from the address and from nowhere else.** A dotted
//! quad or a bracketed v6 literal is verified as an IP address — the engine's
//! leaf must carry the matching `IP:` subject alternative name — and anything
//! else is a DNS name. There is nothing to configure and nothing that can
//! disagree with what was dialled (REMOTE §8: *"the name a client verifies is
//! read off the address it dialled"*).

use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use rustls::ClientConfig;
use rustls::pki_types::ServerName;
use serde_json::Value;

/// The seat's one reading of time, injected.
pub mod clock;
/// How a socket is obtained and first spoken on.
mod dial;
/// What each end of the wire can SPELL, which is not what it must AGREE about.
pub mod edition;
/// The client-side workspaces this box holds elsewhere.
pub mod entries;
/// The wire's framing.
pub mod frame;
/// The version preface.
pub mod hello;
/// The four rungs a dial climbs.
pub mod ladder;
/// The grade, read off this box's own certificate before anything is dialled.
pub mod leaf;
/// One line to one engine, and what is kept of it between asks.
pub(crate) mod line;
/// What the operator carried to this box.
pub mod material;
/// Why an exchange produced no answer, and whether the request crossed.
pub mod reach;
/// The rendezvous material, the two sealed items and the punch.
pub mod rendezvous;
/// The mTLS configuration.
pub mod tls;

use material::Material;
pub use reach::Reach;

/// A seat's end of one wire.
#[derive(Debug)]
pub struct Channel {
    config: Arc<ClientConfig>,
    address: String,
    name: ServerName<'static>,
    /// The rendezvous material the entry carries, or none (§13.4).
    pairing: Option<rendezvous::Pairing>,
    /// The entry's directory: what the run's RAM cache of what worked is
    /// keyed by ([`crate::state::worked`]).
    key: String,
    clock: Arc<dyn clock::Clock>,
    roving: ladder::Roving,
    /// The two files a failed handshake is about, kept so the sentence can name
    /// them (bl-e620). They are read once at [`Channel::open`]; holding the
    /// paths costs nothing and is what turns rustls' own wording into a remedy.
    anchors: PathBuf,
    chain: PathBuf,
    /// **What the engine at the far end could spell, as of the last dial**
    /// (yog's `docs/REMOTE.md` §3.2). A capability rather than an agreement:
    /// the major is settled by the preface's strict equality, and this decides
    /// only whether one control paints a fact or greys it.
    ///
    /// It starts at [`edition::FLOOR`] — the least an engine of this major can
    /// be — so a channel that has never dialled answers about the fields this
    /// major requires and about nothing else. An atomic and not a lock: it is
    /// one integer written by whichever thread dialled last, the crate's locks
    /// live in `crate::state` by rule, and there is no invariant here spanning
    /// two words.
    spelled: AtomicU32,
}

impl Channel {
    /// Open the channel from provisioned material. **Nothing is dialled here**:
    /// a channel is a fact about what this box may say, not about whether an
    /// engine happens to be up.
    pub fn open(m: &Material) -> Result<Self, String> {
        Ok(Self {
            config: tls::client_config(m)?,
            address: m.address.clone(),
            name: server_name(&m.address)?,
            pairing: m.pairing,
            key: m
                .anchors
                .parent()
                .map(|dir| dir.display().to_string())
                .unwrap_or_default(),
            clock: clock::system(),
            roving: ladder::Roving::default(),
            anchors: m.anchors.clone(),
            chain: m.chain.clone(),
            spelled: AtomicU32::new(edition::FLOOR),
        })
    }

    /// The same channel with the roving rungs' knobs and the clock replaced
    /// — the suite's door, so a rendezvous runs against a fake DHT in
    /// milliseconds and a held line's silence is crossed by hand.
    #[cfg(test)]
    pub(crate) fn tuned(self, roving: ladder::Roving, clock: Arc<dyn clock::Clock>) -> Self {
        Self {
            clock,
            roving,
            ..self
        }
    }

    /// **Whether the engine can spell one field of one shape** (yog's
    /// `docs/REMOTE.md` §3.2, [`edition`]).
    ///
    /// The question a control asks before it paints a fact that arrived after
    /// this major was cut: `false` says *this engine cannot say*, which is a
    /// different claim from the field's default and the only reason to ask. A
    /// channel that has not dialled answers for the floor, which is every
    /// field this major requires.
    pub fn spells(&self, shape: &str, key: &str) -> bool {
        edition::spells(self.spelled.load(Ordering::Relaxed), shape, key)
    }

    /// The address it dials.
    pub fn address(&self) -> String {
        self.address.clone()
    }

    /// Send one request and read its whole answer: every frame up to the
    /// terminator. A stream of one is the ordinary answer, and a stream of
    /// several is a follow-class read collected — the same reader, which is
    /// REMOTE §3's *"the streaming form is not a second form"*.
    ///
    /// One `Err` for an unreadable answer and a socket that never opened alike:
    /// both are the same fact to a caller — this cannot be painted, and here is
    /// the sentence. What the [`Reach`] adds is the one fact a sentence cannot
    /// carry and an ACT needs: whether the request crossed (REMOTE §3).
    pub fn ask(&self, request: &Value) -> Result<Vec<Value>, Reach> {
        let mut stream = Vec::new();
        self.follow(request, &mut |frame| {
            stream.push(frame);
            true
        })?;
        Ok(stream)
    }

    /// **Ask, and stay on the line** — the same one request, with each frame
    /// handed over *as it arrives* rather than collected. This is the held
    /// connection a live tail needs, and there is nothing here that
    /// [`ask`](Self::ask) does not already do: the whole difference is that the
    /// caller is given the frames instead of the list, which is why `ask` is
    /// written in terms of this rather than beside it.
    ///
    /// `on_frame` answers whether to stay: `false` ends the read, which is how
    /// a reader whose subject moved stops without a word to the engine
    /// (dropping the connection is the word). `Ok(())` is the engine
    /// terminating the stream — the ordinary end, not an event.
    ///
    /// **Every failure past [`dial`](Self::dial) is [`Reach::Unanswered`]**, and
    /// that is the transport's own seam rather than a judgement: `dial` hands
    /// back a socket with the request already on it, so from here the far end
    /// has the gesture and this end cannot say what it did with it.
    pub fn follow(
        &self,
        request: &Value,
        on_frame: &mut dyn FnMut(Value) -> bool,
    ) -> Result<(), Reach> {
        let mut line = self.dial(request)?;
        while let Some(chunk) =
            line::read(&mut line.tls).map_err(|e| Reach::Unanswered(format!("receive: {e}")))?
        {
            if !on_frame(chunk) {
                return Ok(());
            }
        }
        // The whole answer is in, so a kept line goes back to its pool; a
        // follower that stopped early returned above and dropped its line,
        // which is the word to the engine.
        line.give_back(&self.key, self.clock.now());
        Ok(())
    }
}

/// The name to verify the engine's certificate against, read off the address.
fn server_name(address: &str) -> Result<ServerName<'static>, String> {
    let host = address.rsplit_once(':').map_or(address, |(head, _)| head);
    let host = host.trim_start_matches('[').trim_end_matches(']');
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(ServerName::IpAddress(ip.into()));
    }
    ServerName::try_from(host.to_owned()).map_err(|e| format!("{address}: not a server name: {e}"))
}

#[cfg(test)]
mod tests;
