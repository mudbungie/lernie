//! Lernie library.
//!
//! [`config`] parses and validates the on-disk configuration files
//! described in `docs/ARCHITECTURE.md` §2.2. [`template`] owns the
//! conversation-repo skeleton that the `lernie new` subcommand copies
//! from. [`provider`] holds the HTTP clients that turn a configured
//! provider plus a model-call request into an API call, per §4.1.

pub mod config;
pub mod prompt;
pub mod provider;
pub mod template;
