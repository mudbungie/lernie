//! **The stand-in engine**: the far end of the wire, so the suite can test a
//! channel against something that speaks the protocol rather than against a
//! mock of the seat's own beliefs about it.
//!
//! It listens, which is the one thing a seat must never do — and that is
//! precisely why it is here and not in the crate proper. What it stands in for
//! is yog: a real listener, a real mTLS handshake requiring a client
//! certificate the operator CA issued, a real version preface, and answers
//! framed the way yog's `docs/REMOTE.md` §3 frames them.
//!
//! **It is scripted, one answer per connection**, because a seat dials per ask
//! (REMOTE §3: *"the seat polls"*). So a test says what the engine answers the
//! first dial, the second, and so on.
//!
//! **One of the things it can be told to answer is nothing at all**
//! ([`Answer::Hangup`], bl-3969). That is the seam yog's own bl-d1f1 recorded
//! not having: *"The wire path's window is not drivable without a way to drop a
//! connection mid-answer, which is itself a finding — the path with the worst
//! recovery story is the one with no test seam for it."* A hangup reads the
//! request and then closes without the terminator, which is exactly the shape
//! REMOTE §3's IN DOUBT is made of: the gesture crossed, and no answer came
//! back.
//!
//! **It records every frame it is handed, in order** — the version preface and
//! then the request, per connection. That is how a test asserts what the seat
//! *said* rather than only what it did with the reply, and it is the only
//! witness that the preface goes out in the same breath as the request (REMOTE
//! §3: both ends write before either reads). It is also the witness for §8.2's
//! rename: the frame this end recorded is the envelope as the HOST reads it.

use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex, PoisonError};

use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::{ServerConfig, ServerConnection, StreamOwned};
use serde_json::{Value, json};

use crate::channel::{frame, material, tls};

/// How many frames a seat writes per connection: its preface, then its request.
const FRAMES_IN: usize = 2;

/// **What the stand-in does on one connection.**
///
/// A plain `Vec<Value>` converts into the ordinary arm, so every scripted
/// answer written before this type existed still reads as one — the new arm is
/// the one a test has to name.
pub(crate) enum Answer {
    /// The frames, then the terminator: an answer, ended the way REMOTE §3 ends
    /// one.
    Frames(Vec<Value>),
    /// **Read the request and close, saying nothing.** No frames and no
    /// terminator, so the seat's reader meets an EOF where a frame belongs.
    Hangup,
}

impl From<Vec<Value>> for Answer {
    fn from(frames: Vec<Value>) -> Self {
        Self::Frames(frames)
    }
}

/// A listener standing in for yog, and what it was told.
///
/// It answers no address of its own: the address it bound is written into the
/// scratch directory where [`material`](crate::channel::material) reads it, so
/// a test opens a channel exactly the way an operator-provisioned box does.
pub(crate) struct Engine {
    seen: Arc<Mutex<Vec<Value>>>,
}

impl Engine {
    /// Bind loopback, write the bound address into `dir` where
    /// [`material`](crate::channel::material) reads it, and serve one
    /// connection per entry in `script` — answering the n-th dial with the
    /// n-th entry's frames, then the terminator.
    ///
    /// `protocol` is what the engine states as its version, so a test can make
    /// the two ends disagree without a second code path.
    pub(crate) fn start(dir: &Path, protocol: u32, script: Vec<impl Into<Answer>>) -> Self {
        let script: Vec<Answer> = script.into_iter().map(Into::into).collect();
        let config = server_config(dir);
        let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback port");
        let address = listener.local_addr().expect("bound").to_string();
        std::fs::write(dir.join(material::ADDRESS), address).expect("the address file");
        let seen = Arc::new(Mutex::new(Vec::new()));
        let recorded = Arc::clone(&seen);
        std::thread::spawn(move || {
            for answer in script {
                let Ok((tcp, _)) = listener.accept() else {
                    return;
                };
                serve(&config, tcp, protocol, &answer, &recorded);
            }
        });
        Self { seen }
    }

    /// Every frame it has been handed, in order and across connections.
    pub(crate) fn heard(&self) -> Vec<Value> {
        self.seen
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

/// One connection: state a version, record the two frames the seat writes — its
/// own preface and its request — then answer and terminate.
///
/// Every write ignores its error. A test that makes the seat refuse mid-exchange
/// — a version mismatch, an untrusted anchor — leaves this end writing into a
/// socket that is already gone, and that is the *expected* shape of those tests
/// rather than a failure of the stand-in.
fn serve(
    config: &Arc<ServerConfig>,
    tcp: TcpStream,
    protocol: u32,
    answer: &Answer,
    seen: &Arc<Mutex<Vec<Value>>>,
) {
    let Ok(conn) = ServerConnection::new(Arc::clone(config)) else {
        return;
    };
    let mut tls = StreamOwned::new(conn, tcp);
    let _ = frame::write_value(&mut tls, &json!({ "protocol": protocol }));
    for _ in 0..FRAMES_IN {
        if let Ok(Some(said)) = frame::read_value(&mut tls) {
            seen.lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(said);
        }
    }
    let Answer::Frames(frames) = answer else {
        return;
    };
    for value in frames {
        let _ = frame::write_value(&mut tls, value);
    }
    let _ = frame::write_end(&mut tls);
}

/// The engine's end of the mTLS: present the engine leaf, and require a client
/// certificate the operator CA issued. Requiring one is the point — a stand-in
/// that accepted an anonymous connection would prove nothing about the channel
/// the seat actually opens.
fn server_config(dir: &Path) -> Arc<ServerConfig> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let anchors = tls::anchors(&dir.join(material::ANCHORS)).expect("the operator CA");
    let verifier =
        WebPkiClientVerifier::builder_with_provider(Arc::new(anchors), Arc::clone(&provider))
            .build()
            .expect("a client verifier");
    let chain: Vec<CertificateDer<'static>> =
        CertificateDer::pem_file_iter(dir.join(format!("{}.pem", super::mint::ENGINE)))
            .expect("the engine chain")
            .collect::<Result<_, _>>()
            .expect("the engine chain");
    let key = PrivateKeyDer::from_pem_file(dir.join(format!("{}.key", super::mint::ENGINE)))
        .expect("the engine key");
    Arc::new(
        ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .expect("tls versions")
            .with_client_cert_verifier(verifier)
            .with_single_cert(chain, key)
            .expect("the engine identity"),
    )
}
