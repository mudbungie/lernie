//! **One line to one engine** (yog's `docs/REMOTE.md` §13.4; DESIGN §4.40):
//! the stream a rung of the ladder handed back, how it is read, and what is
//! kept of it once the answer is in.
//!
//! **A punched line is held between asks, and a direct one is not.** First
//! contact over the commons costs seconds — an inbox write, a poll period, a
//! punch window — so the connection it buys is kept and reused, which is the
//! payer §10 of that document waited for. A direct dial costs milliseconds
//! and stays what it always was: one connection per ask.
//!
//! **The engine pings the silence and this end discards the ping.** Between
//! asks the engine writes `{"ping":true}` as a frame every 25 seconds of
//! nothing, to keep both NATs' mappings alive; it is never the start of a
//! reply stream, so [`read`] skips it wherever a frame is read. The engine
//! hangs up after two minutes of silence in all, so a held line is judged
//! gone **one ping before that bound** — the same arithmetic the engine's own
//! `pings_at` runs — rather than reused at the moment the far end is closing
//! it. Judged by the injected clock, so the suite crosses the bound by hand.
//!
//! **And a held line is asked whether it is still open before it is spoken
//! on.** Whatever the socket holds is drawn into rustls without consuming a
//! byte of plaintext — a FIN, a `close_notify`, a reset, or nothing but
//! pings still to be read — so a dropped connection re-enters the ladder at
//! the next ask instead of turning it into an act IN DOUBT. What that cannot
//! see is a peer that vanishes *after* the check, which is REMOTE §3's doubt
//! and is answered the way every lost reply is: by a read.

use std::io::{self, ErrorKind};
use std::net::TcpStream;
use std::time::{Duration, Instant};

use rustls::{ClientConnection, StreamOwned};
use serde_json::{Value, json};

use super::frame;

/// How long a read may find nothing, in all, before the engine is judged
/// gone — the engine's own bound, and the bound it hangs up on.
///
/// It is a bound on the **transport**, not on the wait. An ordinary ask is
/// answered at once; a follow-class read is answered by the engine at the
/// rate the thing being followed writes, so this has to sit comfortably
/// above that cadence or an idle tail would read as a dead channel.
pub(crate) const GONE: Duration = Duration::from_mins(2);
/// How often the engine pings a held line's silence.
pub(crate) const PING: Duration = Duration::from_secs(25);

/// The mTLS stream over one TCP connection.
pub(crate) type Tls = StreamOwned<ClientConnection, TcpStream>;

/// A connection a rung produced, with a request about to go down it.
pub(crate) struct Line {
    pub(crate) tls: Tls,
    /// What the engine stated it could spell on this connection's preface.
    pub(crate) spelled: u32,
    /// Whether the preface is still owed: a stream a rung just opened.
    pub(crate) fresh: bool,
    /// Whether the line is kept once the answer is in: a punched one is.
    pub(crate) kept: bool,
}

impl Line {
    /// A held line taken back up: its preface was exchanged when it was new.
    pub(crate) fn from_held(held: Held) -> Line {
        Line {
            tls: held.tls,
            spelled: held.spelled,
            fresh: false,
            kept: true,
        }
    }

    /// The answer is in: a kept line goes back to the entry's pool, timed
    /// from now; any other is dropped, which is the hang-up.
    pub(crate) fn give_back(self, key: &str, now: Instant) {
        if !self.kept {
            return;
        }
        let held = Held {
            tls: self.tls,
            spelled: self.spelled,
            heard: now,
        };
        crate::state::worked(key, |w| w.held.push(held));
    }
}

/// A line kept between asks.
pub(crate) struct Held {
    tls: Tls,
    spelled: u32,
    /// When the last answer on it was complete.
    heard: Instant,
}

impl Held {
    /// Whether this line is still one the engine would answer on: inside the
    /// silence bound with a ping to spare, and not hung up at the far end.
    pub(crate) fn alive(&mut self, now: Instant) -> bool {
        now.saturating_duration_since(self.heard) + PING < GONE && open(&mut self.tls)
    }
}

/// Draw whatever the socket holds into rustls without reading a byte of
/// plaintext, and say whether the peer is still there. What rustls already
/// holds is judged first — a `close_notify` that rode in behind the last
/// answer is known before the socket is asked for more.
fn open(tls: &mut Tls) -> bool {
    let _ = tls.sock.set_nonblocking(true);
    let verdict = loop {
        match tls.conn.process_new_packets() {
            Ok(state) if state.peer_has_closed() => break false,
            Ok(_) => {}
            Err(_) => break false,
        }
        if !tls.conn.wants_read() {
            break true;
        }
        match tls.conn.read_tls(&mut tls.sock) {
            Ok(0) => break false,
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::WouldBlock => break true,
            Err(_) => break false,
        }
    };
    let _ = tls.sock.set_nonblocking(false);
    verdict
}

/// Read one frame that is not a ping: `Some` a frame of the answer, `None`
/// the terminator. The engine's ping is discarded here, wherever a frame is
/// read, because it is never part of any answer.
pub(crate) fn read(r: &mut dyn io::Read) -> io::Result<Option<Value>> {
    loop {
        match frame::read_value(r)? {
            Some(frame) if frame == ping() => {}
            other => return Ok(other),
        }
    }
}

/// The frame the engine writes into a held line's silence.
pub(crate) fn ping() -> Value {
    json!({"ping": true})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bounds_are_the_engines() {
        assert_eq!(GONE, Duration::from_mins(2));
        assert_eq!(PING, Duration::from_secs(25));
        assert_eq!(ping(), json!({"ping": true}));
    }

    #[test]
    fn a_read_skips_every_ping_and_hands_back_the_frame_or_the_end() {
        let mut bytes = Vec::new();
        frame::write_value(&mut bytes, &ping()).unwrap();
        frame::write_value(&mut bytes, &ping()).unwrap();
        frame::write_value(&mut bytes, &json!({"n": 1})).unwrap();
        frame::write_value(&mut bytes, &ping()).unwrap();
        frame::write_end(&mut bytes).unwrap();
        let mut cursor = std::io::Cursor::new(bytes);
        assert_eq!(read(&mut cursor).unwrap(), Some(json!({"n": 1})));
        assert_eq!(read(&mut cursor).unwrap(), None);
    }
}
