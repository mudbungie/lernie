//! Lernie library.
//!
//! [`config`] parses and validates the configuration files carried by a
//! config commit, described in `docs/ARCHITECTURE.md` §2.2.
//! [`harness_root`] resolves the installation-global harness root —
//! split by XDG lifetime into a config root (`models.yaml`,
//! `workflows/`) and a data root (`workspaces/` plus the `skills/` and
//! `tools/` pools), collapsed to one directory by `LERNIE_HOME` (ARCH
//! §2.2). [`install`] founds that harness root — the idempotent
//! seed-if-absent `lernie prime` verb `make install` invokes (§2.2).
//! [`workspace`] owns the workspace physical model (§2.2–§2.3):
//! the bare `repo.git`, the `config/*` / `agents/*` ref namespaces, and
//! governing-config resolution. [`template`] owns the config-commit
//! skeleton that `lernie new` authors the first config commit from.
//! [`provider`] holds the response-segment classifier over brazen's
//! `v=1` event vocabulary; the provider adapter itself is brazen's
//! external `bz` binary, exec'd per attempt (§4.4).

/// [`archive`] bundles an agent subtree into one `git bundle` plus the
/// `steps/` and `inbox/` slices, and replays it into a scratch workspace
/// for inspection with the ordinary frontend (ARCH §9.2).
pub mod archive;
pub mod config;
pub mod harness_root;
pub mod install;
pub mod prompt;
pub mod provider;
pub mod skill;
pub mod template;
pub mod workspace;
