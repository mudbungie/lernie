//! Lernie library.
//!
//! [`config`] parses and validates the on-disk configuration files
//! described in `docs/ARCHITECTURE.md` §2.2. [`harness_root`] resolves
//! the installation-global directory (`LERNIE_HOME` or `~/.lernie/`,
//! ARCH §2.2) that holds the global `models.yaml`, the `workflows/`,
//! `tools/`, and `skills/` trees, and the `agents/` per-profile
//! skeletons. [`template`] owns the conversation-repo skeleton that the
//! `lernie new` subcommand copies from. [`provider`] holds the
//! response-segment classifier over brazen's `v=1` event vocabulary;
//! the provider adapter itself is brazen's external `bz` binary, exec'd
//! per attempt (§4.4).

pub mod config;
pub mod harness_root;
pub mod prompt;
pub mod provider;
pub mod template;
