//! Tests for the v0.4 Phase 2 `dispatch` built-in. Split by axis so
//! each file stays under the 300-line cap.
//!
//! - [`fixtures`]: stub [`super::EnvLookup`] + [`super::Spawner`] and
//!   the on-disk fake-conv-repo helper, shared across happy/errors.
//! - [`happy`]: the success path — input → spawn-args → handle JSON.
//! - [`errors`]: every failure variant of [`super::Error`].

mod errors;
mod fixtures;
mod happy;
