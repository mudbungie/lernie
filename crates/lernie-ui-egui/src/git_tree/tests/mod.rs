//! Tests for the git-tree view-model.
//!
//! Tests are split by concern: [`fixture`] owns the shared repo-
//! building helpers, [`unit`] drives the pure-function layer
//! (detection, parsing, preview extraction), [`repo`] covers the
//! end-to-end [`super::GitTree::from_repo`] flow against real
//! tempdir-backed git repos, and [`render`] exercises the egui
//! rendering stubs.

mod fixture;
mod render;
mod repo;
mod unit;
