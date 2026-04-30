//! Tests for the v0.4 Phase 3 `await` built-in. Split by axis so each
//! file stays under the 300-line cap.
//!
//! - [`fixtures`]: shared stubs ([`super::EnvLookup`] /
//!   [`super::Sleeper`]) and the on-disk conv-repo helper that boots
//!   a real `git init -b main` in a tempdir.
//! - [`merged`]: happy path — sub merged into parent, summary read.
//! - [`stopped`]: §4.4 `error` event in the latest response.json.
//! - [`conflicted`]: `refs/lernie/conflicted/<handle>` ref present.
//! - [`errors`]: every failure variant of [`super::Error`].

mod conflicted;
mod errors;
mod fixtures;
mod merged;
mod stopped;
