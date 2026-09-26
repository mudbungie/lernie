//! **The far end of a punched wire**, so the ladder can be tested against
//! something that speaks the protocol on a HELD line: an engine that serves
//! every request that lands on one connection, pings its silence, and ends
//! the connection every way a real one can end (DESIGN §4.40).
//!
//! It stands in two ways. [`Roving::listen`] serves what lands on a listener
//! the test bound — the endpoint a presence item names, where the seat's
//! SYN lands. [`Roving::call_back`] is the engine's inbox loop in miniature:
//! it watches a fake DHT node's store for a call under the pairing, opens
//! it, and connects TO the seat's endpoints — the case where the engine's
//! SYN lands on the seat's own punch listener.
//!
//! Like [`engine`](super::engine) it listens, which a seat never does, and
//! that is why it lives here.

use std::collections::VecDeque;
use std::io::Write;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use rustls::{ServerConfig, ServerConnection, StreamOwned};
use serde_json::{Value, json};

use crate::channel::frame;
use crate::channel::line;
use crate::channel::rendezvous::Pairing;
use crate::channel::rendezvous::item::Call;
use crate::dht::Mutable;

/// What the connection does once an answer is written.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Fate {
    /// Hold the line, as a real engine does.
    Stay,
    /// Hold the line and ping its silence this many times, as a real engine
    /// does at 25-second intervals — here at once, so the pings sit in the
    /// seat's socket before its next ask.
    Pinged(usize),
    /// Close without a word: a FIN and nothing else.
    Fin,
    /// Say `close_notify`, then close — the engine's own hang-up.
    Farewell,
    /// Reset: linger zero, so the peer reads a connection reset.
    Reset,
    /// Write bytes that are not TLS, then hold — a peer gone wrong.
    Garbage,
}

/// One request's answer: pings written into the line first, the frames,
/// and what the connection does afterwards.
#[derive(Clone)]
pub(crate) struct Reply {
    pub(crate) pings: usize,
    pub(crate) frames: Vec<Value>,
    pub(crate) fate: Fate,
}

impl Reply {
    /// The ordinary shape: one frame that says yes, the line held.
    pub(crate) fn yes() -> Reply {
        Reply {
            pings: 0,
            frames: vec![json!({"ok": true, "kind": "workspaces"})],
            fate: Fate::Stay,
        }
    }

    pub(crate) fn then(self, fate: Fate) -> Reply {
        Reply { fate, ..self }
    }

    pub(crate) fn pinged(self, pings: usize) -> Reply {
        Reply { pings, ..self }
    }
}

type Script = Arc<Mutex<VecDeque<Reply>>>;

/// The stand-in, and what it was told.
pub(crate) struct Roving {
    seen: Arc<Mutex<Vec<Value>>>,
    connections: Arc<AtomicUsize>,
}

impl Roving {
    /// Serve every connection that lands on `listener`, one [`Reply`] per
    /// request across all of them, in order.
    pub(crate) fn listen(dir: &Path, listener: TcpListener, script: Vec<Reply>) -> Roving {
        let config = super::engine::server_config(dir);
        let roving = Roving::new();
        let (script, seen, connections) = roving.handles(script);
        std::thread::spawn(move || {
            while let Ok((tcp, _)) = listener.accept() {
                connections.fetch_add(1, Ordering::Relaxed);
                let (config, script, seen) =
                    (Arc::clone(&config), Arc::clone(&script), Arc::clone(&seen));
                std::thread::spawn(move || serve(&config, tcp, &script, &seen));
            }
        });
        roving
    }

    /// Watch `store` for a call under `pairing`, connect to what it names,
    /// and serve. One call, one connection: the engine remembers the nonce.
    pub(crate) fn call_back(
        dir: &Path,
        store: Arc<Mutex<Vec<Mutable>>>,
        pairing: Pairing,
        script: Vec<Reply>,
    ) -> Roving {
        let config = super::engine::server_config(dir);
        let roving = Roving::new();
        let (script, seen, connections) = roving.handles(script);
        std::thread::spawn(move || {
            let inbox = pairing.inbox_keypair().unwrap().public();
            let call = loop {
                let found = store
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .iter()
                    .find(|item| item.key == inbox)
                    .and_then(|item| Call::open(&pairing.seal_key(), &item.value));
                if let Some(call) = found {
                    break call;
                }
                std::thread::sleep(Duration::from_millis(20));
            };
            for endpoint in call.endpoints {
                if let Ok(tcp) = TcpStream::connect(endpoint) {
                    connections.fetch_add(1, Ordering::Relaxed);
                    serve(&config, tcp, &script, &seen);
                    return;
                }
            }
        });
        roving
    }

    fn new() -> Roving {
        Roving {
            seen: Arc::new(Mutex::new(Vec::new())),
            connections: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn handles(&self, script: Vec<Reply>) -> (Script, Arc<Mutex<Vec<Value>>>, Arc<AtomicUsize>) {
        (
            Arc::new(Mutex::new(script.into())),
            Arc::clone(&self.seen),
            Arc::clone(&self.connections),
        )
    }

    /// Every frame it has been handed, in order and across connections.
    pub(crate) fn heard(&self) -> Vec<Value> {
        self.seen
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// How many connections have reached it.
    pub(crate) fn connections(&self) -> usize {
        self.connections.load(Ordering::Relaxed)
    }
}

/// One connection: state a version, then per request record what the seat
/// wrote and answer the next [`Reply`]. The seat's own preface is the first
/// frame of a fresh connection and is recorded like any other.
fn serve(
    config: &Arc<ServerConfig>,
    tcp: TcpStream,
    script: &Script,
    seen: &Arc<Mutex<Vec<Value>>>,
) {
    let Ok(conn) = ServerConnection::new(Arc::clone(config)) else {
        return;
    };
    let mut tls = StreamOwned::new(conn, tcp);
    let _ = frame::write_value(
        &mut tls,
        &json!({ "protocol": crate::channel::hello::PROTOCOL }),
    );
    let mut preface_owed = true;
    while let Ok(Some(said)) = frame::read_value(&mut tls) {
        seen.lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(said);
        if preface_owed {
            preface_owed = false;
            continue;
        }
        let Some(reply) = script
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .pop_front()
        else {
            return;
        };
        for _ in 0..reply.pings {
            let _ = frame::write_value(&mut tls, &line::ping());
        }
        for value in &reply.frames {
            let _ = frame::write_value(&mut tls, value);
        }
        let _ = frame::write_end(&mut tls);
        match reply.fate {
            Fate::Stay => {}
            Fate::Pinged(n) => {
                for _ in 0..n {
                    let _ = frame::write_value(&mut tls, &line::ping());
                }
            }
            Fate::Fin => return,
            Fate::Farewell => {
                tls.conn.send_close_notify();
                let _ = tls.flush();
                let _ = tls.sock.shutdown(Shutdown::Both);
                return;
            }
            // A reset discards what the peer has not yet read, and bytes that
            // are not TLS poison the record the answer rode in with — so both
            // wait for the answer to be taken before the line goes wrong.
            Fate::Reset => {
                std::thread::sleep(Duration::from_millis(100));
                let _ = socket2::SockRef::from(&tls.sock).set_linger(Some(Duration::ZERO));
                return;
            }
            Fate::Garbage => {
                std::thread::sleep(Duration::from_millis(100));
                let _ = tls.sock.write_all(b"this is not a TLS record");
                std::thread::sleep(Duration::from_secs(1));
                return;
            }
        }
    }
}
