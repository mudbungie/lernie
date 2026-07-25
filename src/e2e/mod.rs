//! In-crate end-to-end tests that spawn the cargo-built `lernie` binary
//! *and* reach into private machinery to build fixtures (authoring a
//! config commit, driving `RealGit`, bundling/replaying an archive,
//! holding the executor lock). Migrated in from `tests/` when the library
//! surface was narrowed to [`crate::cmd`] (§3.4): once those helpers went
//! private, an external integration test could no longer name them, so
//! these live in-crate as `#[cfg(test)]` modules. The binary they spawn is
//! resolved via [`crate::test_support::lernie_binary`].
//!
//! Tests that only spawn the binary (no private fixture) stay in `tests/`.

mod advance_cli;
mod bundle_replay_cli;
mod message_cli;
mod prompt_end_to_end;
mod prompt_retry;
mod scan_cli;
mod stop_cli;
mod stop_common;
mod stop_idempotence;
