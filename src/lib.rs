//! Lernie library.
//!
//! [`config`] parses and validates the on-disk configuration files
//! described in `docs/ARCHITECTURE.md` §2.2. [`harness_root`] resolves
//! the installation-global harness root — split by XDG lifetime into a
//! config root (`models.yaml`, `workflows/`) and a data root
//! (`conversations/` plus the `agents/`, `skills/`, and `tools/` pools),
//! collapsed to one directory by `LERNIE_HOME` (ARCH §2.2).
//! [`template`] owns the conversation-repo skeleton that the
//! `lernie new` subcommand copies from. [`provider`] holds the
//! response-segment classifier over brazen's `v=1` event vocabulary;
//! the provider adapter itself is brazen's external `bz` binary, exec'd
//! per attempt (§4.4).

pub mod config;
pub mod harness_root;
pub mod prompt;
pub mod provider;
pub mod skill;
pub mod template;
