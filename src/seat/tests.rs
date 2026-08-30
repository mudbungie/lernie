//! The seat's suite, and the scaffolding its two halves share.
//!
//! Split at the line cap along the seam the module itself has: [`routing`] is
//! which engine a gesture reaches and what it carries there, [`listing`] is
//! what this box says it holds without dialling any of it.

use super::{ask, listing, route};
use crate::channel::entries::{ENTRIES, WIRE};
use crate::channel::hello::PROTOCOL;
use crate::test_support::engine::Engine;
use crate::test_support::{Scratch, mint};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// A data root, with an engine standing behind one channel of it.
///
/// `at` is the directory under the root the material goes in — the flat root
/// (`wire`) or one entry (`wire/workspaces/<leaf>`) — so one helper stands both
/// arrangements up.
fn wired(scratch: &Scratch, at: &Path, script: Vec<Vec<Value>>) -> Engine {
    let dir = scratch.path().join(at);
    std::fs::create_dir_all(&dir).expect("mkdir");
    mint::material(&dir);
    Engine::start(&dir, PROTOCOL, script)
}

/// The flat root's path under a data root.
fn flat() -> PathBuf {
    PathBuf::from(WIRE)
}

/// One entry's path under a data root.
fn entry(leaf: &str) -> PathBuf {
    flat().join(ENTRIES).join(leaf)
}

/// An answer of one frame that says yes.
fn yes() -> Value {
    json!({"ok": true, "kind": "workspaces"})
}

/// What the box says it holds, without dialling any of it.
mod listing;
/// Which engine a gesture reaches and what it carries there.
mod routing;
