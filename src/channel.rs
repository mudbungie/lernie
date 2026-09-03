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
//! **The engine's name comes from the address and from nowhere else.** A dotted
//! quad or a bracketed v6 literal is verified as an IP address — the engine's
//! leaf must carry the matching `IP:` subject alternative name — and anything
//! else is a DNS name. There is nothing to configure and nothing that can
//! disagree with what was dialled (REMOTE §8: *"the name a client verifies is
//! read off the address it dialled"*).

use std::net::{IpAddr, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, StreamOwned};
use serde_json::Value;

/// The client-side workspaces this box holds elsewhere.
pub mod entries;
/// The wire's framing.
pub mod frame;
/// The version preface.
pub mod hello;
/// The grade, read off this box's own certificate before anything is dialled.
pub mod leaf;
/// What the operator carried to this box.
pub mod material;
/// Why an exchange produced no answer, and whether the request crossed.
pub mod reach;
/// The mTLS configuration.
pub mod tls;

use material::Material;
pub use reach::Reach;

/// How long one read may wait before the channel is judged gone.
///
/// It is a bound on the **transport**, not on the wait. An ordinary ask is
/// answered at once; a follow-class read is answered by the engine at the rate
/// the thing being followed writes, so this has to sit comfortably above that
/// cadence or an idle tail would read as a dead channel.
const READ_TIMEOUT: Duration = Duration::from_mins(2);

/// A seat's end of one wire.
#[derive(Debug)]
pub struct Channel {
    config: Arc<ClientConfig>,
    address: String,
    name: ServerName<'static>,
    /// The two files a failed handshake is about, kept so the sentence can name
    /// them (bl-e620). They are read once at [`Channel::open`]; holding the
    /// paths costs nothing and is what turns rustls' own wording into a remedy.
    anchors: PathBuf,
    chain: PathBuf,
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
            anchors: m.anchors.clone(),
            chain: m.chain.clone(),
        })
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
        let mut tls = self.dial(request)?;
        while let Some(chunk) =
            frame::read_value(&mut tls).map_err(|e| Reach::Unanswered(format!("receive: {e}")))?
        {
            if !on_frame(chunk) {
                return Ok(());
            }
        }
        Ok(())
    }

    /// Connect, handshake and send. The TLS handshake happens inside the first
    /// write, and the one frame read here is the engine's version preface — so
    /// what this hands back is a socket with a request on it and no *answer*
    /// yet read.
    ///
    /// **Both ends state a version before either reads** (REMOTE §3), and the
    /// request goes out in the same breath as this end's preface — so
    /// confirming the engine's costs no round trip, and a mismatch refuses
    /// before a frame of the answer is decoded.
    ///
    /// **It is also where the doubt begins** (REMOTE §3, bl-3969). Everything
    /// refused here is [`Reach::Unsent`] and nothing crossed — a socket that
    /// would not open, a handshake that did not verify, a write that failed, a
    /// peer of another protocol, which refuses *"before any gesture is
    /// decoded"*. The one exception is a preface this end could not READ, which
    /// [`hello::confirm`] classes for itself: the request went out in the same
    /// breath as this end's preface, so a connection that broke before the
    /// engine's answer got back may have broken after it ran the gesture.
    fn dial(&self, request: &Value) -> Result<StreamOwned<ClientConnection, TcpStream>, Reach> {
        let tcp = TcpStream::connect(&self.address)
            .and_then(|tcp| tcp.set_read_timeout(Some(READ_TIMEOUT)).map(|()| tcp))
            .map_err(|e| Reach::Unsent(format!("connect {}: {e}", self.address)))?;
        let conn = ClientConnection::new(Arc::clone(&self.config), self.name.clone())
            .map_err(|e| Reach::Unsent(format!("tls {}: {e}", self.address)))?;
        let mut tls = StreamOwned::new(conn, tcp);
        hello::state(&mut tls).map_err(|e| Reach::Unsent(self.wrote(&e)))?;
        frame::write_value(&mut tls, request).map_err(|e| Reach::Unsent(self.wrote(&e)))?;
        hello::confirm(&mut tls)?;
        Ok(tls)
    }

    /// **What a failed write says.**
    ///
    /// The TLS handshake happens inside the first write, so an error here is
    /// usually not a socket at all — it is the two ends failing to accept each
    /// other's certificates, and rustls says so in its own words: *"invalid
    /// peer certificate: UnknownIssuer"*, which names no file and no act
    /// (bl-e620, driven live: a wrong anchor produced that and nothing else,
    /// anywhere).
    ///
    /// So the one class that is always a fact about **this box's own material**
    /// carries the remedy, and every other write error is still said in the
    /// transport's own words: a certificate fault is read off the typed
    /// `rustls::Error` rather than off its wording, so a rewritten message
    /// cannot silently stop matching.
    fn wrote(&self, e: &std::io::Error) -> String {
        let Some(rustls::Error::InvalidCertificate(fault)) = e
            .get_ref()
            .and_then(|inner| inner.downcast_ref::<rustls::Error>())
        else {
            return format!("send: {e}");
        };
        format!(
            "the handshake with {} did not verify ({fault:?}): {} must hold the anchors of THAT engine's CA, and {} must be a leaf that CA issued. Both are carried here by hand; the seat mints nothing",
            self.address,
            self.anchors.display(),
            self.chain.display()
        )
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
