//! **A data root with an engine behind one of its channels** — the one fixture
//! for standing a real wire up, shared by the seat's suite and the off-frame
//! threads'.
//!
//! One home, because both suites need the same three things: a scratch root, an
//! operator's material minted into one directory under it, and a listener
//! answering a script. Two copies of that would be two ideas of what a
//! provisioned box looks like.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::channel::entries::{ENTRIES, WIRE};
use crate::channel::hello::PROTOCOL;
use crate::test_support::engine::Engine;
use crate::test_support::{Scratch, mint};

/// A data root, with an engine standing behind one channel of it.
///
/// `at` is the directory under the root the material goes in — the flat root
/// (`wire`) or one entry (`wire/workspaces/<leaf>`) — so one helper stands both
/// arrangements up.
pub(crate) fn wired(scratch: &Scratch, at: &Path, script: Vec<Vec<Value>>) -> Engine {
    let dir = scratch.path().join(at);
    std::fs::create_dir_all(&dir).expect("mkdir");
    mint::material(&dir);
    Engine::start(&dir, PROTOCOL, script)
}

/// The flat root's path under a data root.
pub(crate) fn flat() -> PathBuf {
    PathBuf::from(WIRE)
}

/// One entry's path under a data root.
pub(crate) fn entry(leaf: &str) -> PathBuf {
    flat().join(ENTRIES).join(leaf)
}

/// An answer of one frame that says yes.
pub(crate) fn yes() -> Value {
    json!({"ok": true, "kind": "workspaces"})
}
