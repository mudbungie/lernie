//! **How a socket is obtained and first spoken on** — the transport half of
//! [`Channel`], split from the root at the design-time budget on the seam
//! the root's doc already draws: that file is what a channel IS and what it
//! can be asked, this is what happens between a request and the first frame
//! of its answer.
//!
//! **The entry's address list is dialled in order** (yog's `docs/REMOTE.md`
//! §15.3; DESIGN §4.49). The endpoint that worked earlier in this run is
//! tried first ([`crate::state::worked`], RAM only), then every other in the
//! order the file states. An endpoint is passed over when its socket will
//! not open *or* its handshake will not verify — a private address dialled
//! from somebody else's network may well be answered, and whatever answers
//! there cannot present the operator's CA — and nothing has crossed in
//! either case, so falling through repeats no gesture.

use std::net::{TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use rustls::{ClientConnection, StreamOwned};
use serde_json::Value;

use super::{Channel, Endpoint, Reach, frame, hello};

/// How long a read may find nothing before the server is judged gone — the
/// server's own bound, and the bound it hangs up on.
///
/// It is a bound on the **transport**, not on the wait. An ordinary ask is
/// answered at once; a follow-class read is answered by the server at the
/// rate the thing being followed writes, so this has to sit comfortably
/// above that cadence or an idle tail would read as a dead channel.
pub(crate) const GONE: Duration = Duration::from_mins(2);

/// How long one resolved address is given to answer a SYN. Bounded because a
/// network that drops the SYN would otherwise cost minutes before the next
/// endpoint in the list is tried.
const CONNECT: Duration = Duration::from_secs(5);

/// The mTLS stream over one TCP connection.
pub(crate) type Tls = StreamOwned<ClientConnection, TcpStream>;

impl Channel {
    /// Connect, handshake and send. The TLS handshake happens inside the first
    /// write, and the one frame read here is the server's version preface — so
    /// what this hands back is a socket with a request on it and no *answer*
    /// yet read.
    ///
    /// **Both ends state a version before either reads** (REMOTE §3), and the
    /// request goes out in the same breath as this end's preface — so
    /// confirming the server's costs no round trip, and a mismatch refuses
    /// before a frame of the answer is decoded.
    ///
    /// **It is also where the doubt begins** (REMOTE §3, bl-3969). Everything
    /// refused here is [`Reach::Unsent`] and nothing crossed — a socket that
    /// would not open, a handshake that did not verify, a write that failed, a
    /// peer of another protocol, which refuses *"before any gesture is
    /// decoded"*. The one exception is a preface this end could not READ, which
    /// [`hello::confirm`] classes for itself: the request went out in the same
    /// breath as this end's preface, so a connection that broke before the
    /// server's answer got back may have broken after it ran the gesture.
    ///
    /// **Every endpoint that refused is named**, each in its own sentence, so
    /// a list that reached nothing says why of every line in it.
    pub(super) fn dial(&self, request: &Value) -> Result<Tls, Reach> {
        let worked = crate::state::worked(&self.key, |worked| worked.clone());
        let mut order: Vec<&Endpoint> = self.endpoints.iter().collect();
        order.sort_by_key(|at| Some(&at.address) != worked.as_ref());
        let mut refusals = Vec::new();
        for at in order {
            match self.send(at, request) {
                Ok(mut tls) => {
                    crate::state::worked(&self.key, |worked| *worked = Some(at.address.clone()));
                    let spelled = hello::confirm(&mut tls)?;
                    self.spelled.store(spelled, Ordering::Relaxed);
                    return Ok(tls);
                }
                Err(refusal) => refusals.push(refusal),
            }
        }
        Err(Reach::Unsent(refusals.join("; ")))
    }

    /// One endpoint: the socket, the mTLS over it verifying the name read off
    /// that same endpoint, and this end's preface and request down it.
    fn send(&self, at: &Endpoint, request: &Value) -> Result<Tls, String> {
        let refused = |e| format!("connect {}: {e}", at.address);
        let tcp = connect(&at.address).map_err(refused)?;
        tcp.set_read_timeout(Some(GONE)).map_err(refused)?;
        let conn = ClientConnection::new(Arc::clone(&self.config), at.name.clone())
            .map_err(|e| format!("tls {}: {e}", at.address))?;
        let mut tls = StreamOwned::new(conn, tcp);
        hello::state(&mut tls)
            .and_then(|()| frame::write_value(&mut tls, request))
            .map_err(|e| self.wrote(&at.address, &e))?;
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
    pub(super) fn wrote(&self, address: &str, e: &std::io::Error) -> String {
        let Some(rustls::Error::InvalidCertificate(fault)) = e
            .get_ref()
            .and_then(|inner| inner.downcast_ref::<rustls::Error>())
        else {
            return format!("send {address}: {e}");
        };
        format!(
            "the handshake with {address} did not verify ({fault:?}): {} must hold the anchors of THAT server's CA, and {} must be a leaf that CA issued. `lernie register` files both from an offer; otherwise both are carried here by hand",
            self.anchors.display(),
            self.chain.display()
        )
    }
}

/// A connect that gives each address `endpoint` resolves to at most
/// [`CONNECT`], and answers the last refusal where none of them took it.
pub(crate) fn connect(endpoint: &str) -> std::io::Result<TcpStream> {
    let mut refusal = std::io::Error::other("resolved to no address");
    for addr in endpoint.to_socket_addrs()? {
        match TcpStream::connect_timeout(&addr, CONNECT) {
            Ok(tcp) => return Ok(tcp),
            Err(e) => refusal = e,
        }
    }
    Err(refusal)
}
