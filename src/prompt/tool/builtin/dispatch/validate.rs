//! Role validation for the dispatch built-in (ARCH §2.2, §4.3): the
//! role must be declared in the calling branch's governing config
//! commit, with its soul present in the same tree. Split from
//! [`super`] for the per-file line cap.

use super::{Error, PER_REPO_PROVIDERS_FILE, SOUL_SUFFIX};
use crate::config::PerRepoProviders;
use crate::prompt::SOULS_DIR;
use crate::template::RealGit;
use crate::workspace;
use std::path::{Path, PathBuf};

/// Confirm the role is defined in the `providers.yaml` of the calling
/// branch's **governing config commit** (`roles:` block, ARCH §2.2,
/// §4.3) AND that its soul exists at `souls/<role>.md` in the same
/// tree (§4.3 — no path override). Control is never read from a
/// worktree file (§2.2). Both checks land before the spawn so we fail
/// with a clean typed error instead of a noisy subprocess exit on a
/// doomed call.
pub(super) fn validate_role(repo: &Path, branch: &str, role: &str) -> Result<(), Error> {
    let git = RealGit::new();
    let gov = |source| Error::GoverningConfig {
        branch: branch.to_string(),
        source,
    };
    let commit = workspace::governing_config(repo, branch, &git).map_err(gov)?;
    let providers_raw =
        workspace::show_control(repo, &commit, PER_REPO_PROVIDERS_FILE, &git).map_err(gov)?;
    let origin = PathBuf::from(format!("{commit}:{PER_REPO_PROVIDERS_FILE}"));
    let providers = PerRepoProviders::parse(&providers_raw, &origin)?;
    if !providers.roles.contains_key(role) {
        return Err(Error::RoleMissing {
            role: role.to_string(),
            path: origin,
        });
    }
    let soul_rel = format!("{SOULS_DIR}/{role}{SOUL_SUFFIX}");
    if !workspace::control_exists(repo, &commit, &soul_rel, &git) {
        return Err(Error::SoulMissing {
            path: PathBuf::from(format!("{commit}:{soul_rel}")),
        });
    }
    Ok(())
}
