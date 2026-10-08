//! Scaffolding the suite shares, and nothing production reads. Compiled only
//! under `cfg(test)`.
//!
//! Two things live here that live nowhere else in the crate, and both are
//! deliberate:
//!
//! - **The certificate mint** ([`mint`]). The seat mints nothing (yog's
//!   `docs/REMOTE.md` §1.4) — the operator issues a pair on the box that holds
//!   the CA and carries it here by hand — so the suite has to perform that act
//!   on the operator's behalf before it can open a channel at all. It is
//!   `cfg(test)`, it shells to the tool an operator would use, and **no
//!   certificate is ever committed**: a fixture key in a tree is a private key
//!   in a repository, which is the exact class `make leak-scan` refuses.
//! - **The stand-in engine** ([`engine`]), which LISTENS — the one thing a seat
//!   must never do. That is precisely why it is here and not in the crate
//!   proper.

use std::net::{Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The suite's clock: an offset the test advances by hand.
pub(crate) mod clock;
/// The one walk over the wire conformance corpus, shared by both replays.
pub(crate) mod corpus;
/// The far end of the wire, so a channel can be tested against something that
/// speaks the protocol.
pub(crate) mod engine;
/// The operator's out-of-channel act, performed by the suite.
pub(crate) mod mint;
/// The far end of a PUNCHED wire: an engine that serves a held line, or
/// that calls back what the seat wrote to its inbox.
pub(crate) mod roving;
/// The window's fixtures, and the two ways a test looks at one.
pub(crate) mod window;
/// A data root with an engine behind one of its channels.
pub(crate) mod wire;

/// How many scratch directories this process has minted, so two tests running
/// at once never name one directory.
static NEXT: AtomicUsize = AtomicUsize::new(0);

/// A throwaway directory, removed when it drops.
///
/// Hand-rolled rather than a crate, because the dependency set is closed
/// (`Cargo.toml`'s approval comment): a scratch directory is a `create_dir_all`
/// and a `remove_dir_all`, and a test-only crate is still a crate in the
/// lockfile, the licence audit and the supply chain.
pub(crate) struct Scratch {
    path: PathBuf,
}

impl Scratch {
    /// Make one, under the platform's temporary directory.
    pub(crate) fn new() -> Self {
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("lernie-test-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&path).expect("a scratch directory");
        Self { path }
    }

    /// The directory itself.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    /// A path inside it. Named, not made — the caller decides whether the
    /// thing at it should exist.
    pub(crate) fn join(&self, leaf: &str) -> PathBuf {
        self.path.join(leaf)
    }

    /// A directory inside it, made.
    pub(crate) fn dir(&self, leaf: &str) -> PathBuf {
        let path = self.join(leaf);
        std::fs::create_dir_all(&path).expect("a scratch subdirectory");
        path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// A loopback TCP address nothing answers, for as long as the process runs: a
/// socket bound and never listening, so a SYN to it is reset, and held so the
/// kernel never hands its port to anyone else.
///
/// Binding a port and dropping it is not this. The number goes back to the
/// kernel's pool, and the next `bind(0)` — another test's listener or punch
/// port, in this process or a concurrent one — may be handed it, so the "dead"
/// address answers as somebody else: a punch aimed at it once handshook with a
/// stranger's punch (bl-73f2). No reuse flag is set, so no other socket can
/// share it.
pub(crate) fn dead() -> SocketAddr {
    static DEAD: OnceLock<(socket2::Socket, SocketAddr)> = OnceLock::new();
    DEAD.get_or_init(|| {
        let socket = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::STREAM, None)
            .expect("a TCP socket");
        socket
            .bind(&SocketAddr::from((Ipv4Addr::LOCALHOST, 0)).into())
            .expect("a loopback port");
        let at = socket
            .local_addr()
            .expect("bound")
            .as_socket()
            .expect("inet");
        (socket, at)
    })
    .1
}

#[cfg(test)]
mod tests;
