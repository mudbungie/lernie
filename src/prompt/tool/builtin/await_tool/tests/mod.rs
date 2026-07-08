//! Tests for the v0.4 `await` built-in. Split by axis so each file
//! stays under the 300-line cap.
//!
//! - [`fixtures`]: shared stubs ([`super::EnvLookup`] /
//!   [`super::Sleeper`] / [`crate::prompt::stop::PgidFinder`]) and
//!   the on-disk conv-repo helper that boots a real `git init -b
//!   main` in a tempdir.
//! - [`merged`]: happy path — sub merged into parent, summary read.
//! - [`stopped`]: §4.4 `error` event in the latest response.json
//!   (the v0.4 P3 stopped signature, kept here for the clean-error
//!   path).
//! - [`killed`]: kill-mid-stream stopped signature — non-terminal
//!   last line + no writer in /proc (ARCH §2.9 / §3.5).
//! - [`conflicted`]: `refs/lernie/conflicted/<handle>` ref present.
//! - [`errors`]: every failure variant of [`super::Error`].

mod budget_exhausted;
mod conflicted;
mod errors;
mod fixtures;
mod killed;
mod merged;
mod stopped;
