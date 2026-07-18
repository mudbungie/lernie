//! Open-set role validation (ARCH §4.3): a role is valid iff the
//! branch's governing config commit (§2.2) lists `roles.<name>` in
//! `providers.yaml` **and** carries `souls/<name>.md`. Nothing else
//! mints a role and the harness never enumerates role names.
//!
//! **Single authoritative home** (`docs/PRINCIPLES.md` Single source of
//! truth): this is the one answer to "is this role dispatchable." Both
//! front doors consult it — the model-facing `dispatch` built-in (§2.5,
//! projecting [`Invalid`] onto its own typed error) and the
//! `lernie dispatch <role>` CLI (§3.4, pre-flighting before the fork so
//! a rejected role leaves no branch debris). There is no hard-coded
//! `worker`/`compactor` list anywhere; the closed vocabulary
//! `worker`/`compactor`/`verifier` belongs to the §6 workflow
//! interpreter, not to dispatch validity (§4.3 severability line).

use crate::config::{LoadError, PerRepoProviders};
use crate::prompt::{PER_REPO_PROVIDERS_FILE, SOULS_DIR};
use crate::template::GitRunner;
use crate::workspace;
use std::io;
use std::path::{Path, PathBuf};

/// Why a role is not dispatchable against a branch's governing config
/// commit. Each variant **names the config commit consulted**, so a
/// refusal points at the exact immutable tree that lacks the role.
#[derive(Debug)]
pub enum Invalid {
    /// The `roles:` block of the governing config's `providers.yaml`
    /// does not list the role. `origin` is `<commit>:providers.yaml`.
    RoleMissing { role: String, origin: PathBuf },
    /// The role is listed but its soul is absent from the same tree
    /// (§4.3 — the name is the path, no override). `path` is
    /// `<commit>:souls/<role>.md`.
    SoulMissing { path: PathBuf },
    /// `providers.yaml` parsed but was malformed / legacy (§4.1).
    Config(LoadError),
    /// Deriving the governing config commit (§2.2) or reading a control
    /// file from its tree failed — a defective or absent workspace.
    Governing { branch: String, source: io::Error },
}

impl std::fmt::Display for Invalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RoleMissing { role, origin } => {
                write!(f, "role {role:?} is not defined in {}", origin.display())
            }
            Self::SoulMissing { path } => write!(f, "soul {} does not exist", path.display()),
            Self::Config(e) => write!(f, "providers.yaml: {e}"),
            Self::Governing { branch, source } => {
                write!(f, "governing config for {branch}: {source}")
            }
        }
    }
}

/// Confirm `role` is dispatchable against `branch`'s governing config
/// commit: listed in `providers.yaml` `roles:` **and** carrying
/// `souls/<role>.md` in the same immutable tree (§4.3). Control is read
/// only from the config commit's tree (§2.2), never a worktree file.
/// Both checks precede any fork, so a rejected role leaves no debris.
pub fn validate(repo: &Path, branch: &str, role: &str, git: &dyn GitRunner) -> Result<(), Invalid> {
    let gov = |source| Invalid::Governing {
        branch: branch.to_string(),
        source,
    };
    let commit = workspace::governing_config(repo, branch, git).map_err(gov)?;
    let providers_raw =
        workspace::show_control(repo, &commit, PER_REPO_PROVIDERS_FILE, git).map_err(gov)?;
    let origin = PathBuf::from(format!("{commit}:{PER_REPO_PROVIDERS_FILE}"));
    let providers = PerRepoProviders::parse(&providers_raw, &origin).map_err(Invalid::Config)?;
    if !providers.roles.contains_key(role) {
        return Err(Invalid::RoleMissing {
            role: role.to_string(),
            origin,
        });
    }
    let soul_rel = format!("{SOULS_DIR}/{role}.md");
    if !workspace::control_exists(repo, &commit, &soul_rel, git) {
        return Err(Invalid::SoulMissing {
            path: PathBuf::from(format!("{commit}:{soul_rel}")),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::RealGit;
    use crate::workspace::fixture;

    fn git() -> RealGit {
        RealGit::new()
    }

    /// The default scaffold lists `worker` with `souls/worker.md`, so a
    /// worker off a fresh root validates.
    #[test]
    fn a_config_role_with_its_soul_is_valid() {
        let (_h, ws) = fixture::workspace();
        fixture::spawn_root(&ws, "p1");
        validate(&ws, "p1", "worker", &git()).unwrap();
    }

    /// A third role the config defines — the v0.7 verifier, zero code —
    /// validates exactly like the template roles.
    #[test]
    fn a_third_config_role_is_valid_zero_code() {
        let (_h, ws) = fixture::workspace();
        let yaml = "roles:\n  worker:\n    provider: anthropic\n    model: sonnet\n  \
                    verifier:\n    provider: anthropic\n    model: sonnet\n";
        fixture::amend_config(
            &ws,
            &[("providers.yaml", yaml), ("souls/verifier.md", "v\n")],
        );
        fixture::spawn_root(&ws, "p9");
        validate(&ws, "p9", "verifier", &git()).unwrap();
    }

    #[test]
    fn a_role_absent_from_providers_is_role_missing() {
        let (_h, ws) = fixture::workspace();
        fixture::spawn_root(&ws, "p1");
        let err = validate(&ws, "p1", "ghost", &git()).unwrap_err();
        match &err {
            Invalid::RoleMissing { role, origin } => {
                assert_eq!(role, "ghost");
                assert!(origin.to_string_lossy().ends_with(":providers.yaml"));
            }
            other => panic!("expected RoleMissing, got {other:?}"),
        }
        assert!(err.to_string().contains("is not defined in"));
    }

    #[test]
    fn a_role_listed_without_a_soul_is_soul_missing() {
        let (_h, ws) = fixture::workspace();
        let yaml = "roles:\n  verifier:\n    provider: anthropic\n    model: sonnet\n";
        fixture::amend_config(&ws, &[("providers.yaml", yaml)]);
        fixture::spawn_root(&ws, "p9");
        let err = validate(&ws, "p9", "verifier", &git()).unwrap_err();
        match &err {
            Invalid::SoulMissing { path } => {
                assert!(path.to_string_lossy().ends_with("souls/verifier.md"));
            }
            other => panic!("expected SoulMissing, got {other:?}"),
        }
        assert!(err.to_string().contains("does not exist"));
    }

    #[test]
    fn a_legacy_providers_yaml_is_config_error() {
        let (_h, ws) = fixture::workspace();
        fixture::amend_config(&ws, &[("providers.yaml", "providers: {}\n")]);
        fixture::spawn_root(&ws, "p9");
        let err = validate(&ws, "p9", "worker", &git()).unwrap_err();
        assert!(matches!(err, Invalid::Config(_)), "{err:?}");
        assert!(err.to_string().starts_with("providers.yaml:"));
    }

    #[test]
    fn a_non_workspace_repo_is_a_governing_error() {
        let holder = tempfile::TempDir::new().unwrap();
        let err = validate(holder.path(), "p1", "worker", &git()).unwrap_err();
        match &err {
            Invalid::Governing { branch, .. } => assert_eq!(branch, "p1"),
            other => panic!("expected Governing, got {other:?}"),
        }
        assert!(err.to_string().contains("governing config for p1"));
    }
}
