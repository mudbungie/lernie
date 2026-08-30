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

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The far end of the wire, so a channel can be tested against something that
/// speaks the protocol.
pub(crate) mod engine;
/// The operator's out-of-channel act, performed by the suite.
pub(crate) mod mint;

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

#[cfg(test)]
mod tests;
