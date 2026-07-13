//! Tests for the git-tree view-model.
//!
//! Tests are split by concern: [`fixture`] owns the shared workspace-
//! building helpers, [`unit`] drives the pure-function layer (detection,
//! parsing, preview extraction), [`repo`] covers the end-to-end
//! [`super::GitTree::from_repo`] flow against real tempdir-backed
//! workspaces, and [`render`] exercises the egui rendering. Agent-state
//! coverage (§3.5) lives in [`state_render`] (badge + mark mapping) and
//! [`state_repo`] (end-to-end quiescent/stopped classification against
//! real fixtures); the `live`/`in_flight` states are probe-injected unit
//! tests in `super::state` and `super::lock_probe`.

mod fixture;
mod render;
mod repo;
mod state_render;
mod state_repo;
mod unit;
