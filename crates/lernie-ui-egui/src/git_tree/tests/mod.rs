//! Tests for the git-tree view-model.
//!
//! Tests are split by concern: [`fixture`] owns the shared repo-
//! building helpers, [`unit`] drives the pure-function layer
//! (detection, parsing, preview extraction), [`repo`] covers the
//! end-to-end [`super::GitTree::from_repo`] flow against real
//! tempdir-backed git repos, and [`render`] exercises the egui
//! rendering stubs. Branch-state coverage (bl-de6b) lives in
//! [`state_render`] (badge mapping) and [`state_repo`] (end-to-end
//! classification against real fixtures).

mod fixture;
mod render;
mod repo;
mod state_render;
mod state_repo;
mod unit;
