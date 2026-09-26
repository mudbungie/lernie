//! **How a socket is obtained and first spoken on** — the transport half of
//! [`Channel`], split from the root at the design-time budget on the seam
//! the root's doc already draws: that file is what a channel IS and what it
//! can be asked, this is what happens between a request and the first frame
//! of its answer.
//!
//! Three things, in the order a dial runs them: the ladder is climbed for a
//! [`Line`] ([`super::ladder`]), the preface and the request go down it, and
//! a failed write is turned into the sentence an operator can act on.

use std::net::TcpStream;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use rustls::{ClientConnection, StreamOwned};
use serde_json::Value;

use super::{Channel, Reach, edition, frame, hello, ladder, line};

impl Channel {
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
    ///
    /// **A held line owes no preface** (REMOTE §13.4): it was exchanged when
    /// the line was new, so the request alone goes down it. A held line that
    /// will not take the write is gone — nothing crossed — and is dropped
    /// with the refusal; the next ask climbs the ladder below it.
    pub(super) fn dial(&self, request: &Value) -> Result<line::Line, Reach> {
        let mut line = ladder::climb(self, true).map_err(Reach::Unsent)?;
        if line.fresh {
            hello::state(&mut line.tls).map_err(|e| Reach::Unsent(self.wrote(&e)))?;
        }
        frame::write_value(&mut line.tls, request).map_err(|e| Reach::Unsent(self.wrote(&e)))?;
        if line.fresh {
            line.spelled = hello::confirm(&mut line.tls)?;
        }
        self.spelled.store(line.spelled, Ordering::Relaxed);
        Ok(line)
    }

    /// The inner mTLS over a stream a rung produced — the same configuration,
    /// the same name off the same address, whichever rung it was.
    pub(crate) fn line(&self, tcp: TcpStream, kept: bool) -> Result<line::Line, String> {
        tcp.set_read_timeout(Some(line::GONE))
            .map_err(|e| format!("connect {}: {e}", self.address))?;
        let conn = ClientConnection::new(Arc::clone(&self.config), self.name.clone())
            .map_err(|e| format!("tls {}: {e}", self.address))?;
        Ok(line::Line {
            tls: StreamOwned::new(conn, tcp),
            spelled: edition::FLOOR,
            fresh: true,
            kept,
        })
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
    pub(super) fn wrote(&self, e: &std::io::Error) -> String {
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
