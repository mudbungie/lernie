//! Resolution of the **harness root** (ARCH §2.2).
//!
//! The harness root holds installation-global state outside any one
//! conversation repo: the global `providers.yaml` (ARCH §4.1), and the
//! per-profile `agents/`, `adapters/`, `workflows/`, `tools/`, and
//! `skills/` directories. Conversation repos live underneath at
//! `<root>/conversations/<root-id>/`.
//!
//! `LERNIE_HOME`, when set and non-empty, is used verbatim. Otherwise
//! the root defaults to `~/.lernie/`. Resolution is deliberately not
//! cached — tests scope env-var changes per call, and the cost of one
//! `getenv` per use is irrelevant next to the I/O it gates.

use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use thiserror::Error;

const ENV_VAR: &str = "LERNIE_HOME";
const DEFAULT_DIRNAME: &str = ".lernie";
const MODELS_FILE: &str = "models.yaml";

/// Why [`resolve`] could not produce a path. The only failure is the
/// "no override and no home" pair — every other case yields a path
/// (whether or not the directory exists on disk).
#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("LERNIE_HOME is unset and no home directory is available")]
    NoHome,
}

/// Pure resolver. `override_value` is the literal `LERNIE_HOME` value
/// (or `None` if unset); `home` is the user's home directory (or
/// `None` if unknown). An empty `override_value` falls through to the
/// home-based default — an empty env var would otherwise produce the
/// current working directory's neighbor and is almost never intended.
pub fn resolve_from(override_value: Option<&OsStr>, home: Option<&Path>) -> Result<PathBuf, Error> {
    if let Some(v) = override_value
        && !v.is_empty()
    {
        return Ok(PathBuf::from(v));
    }
    home.map(|h| h.join(DEFAULT_DIRNAME)).ok_or(Error::NoHome)
}

/// Resolve the harness root from the live process environment.
pub fn resolve() -> Result<PathBuf, Error> {
    let env_val = env::var_os(ENV_VAR);
    #[allow(deprecated)] // un-deprecated in Rust 1.86; lint precedes that.
    let home = env::home_dir();
    resolve_from(env_val.as_deref(), home.as_deref())
}

/// Path to the global `models.yaml` within `root` (ARCH §4.2).
pub fn models_path(root: &Path) -> PathBuf {
    root.join(MODELS_FILE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn override_takes_precedence() {
        let p = resolve_from(Some(OsStr::new("/opt/lernie")), Some(Path::new("/home/x"))).unwrap();
        assert_eq!(p, PathBuf::from("/opt/lernie"));
    }

    #[test]
    fn empty_override_falls_through_to_home() {
        let p = resolve_from(Some(OsStr::new("")), Some(Path::new("/home/x"))).unwrap();
        assert_eq!(p, PathBuf::from("/home/x/.lernie"));
    }

    #[test]
    fn unset_override_uses_home_default() {
        let p = resolve_from(None, Some(Path::new("/home/x"))).unwrap();
        assert_eq!(p, PathBuf::from("/home/x/.lernie"));
    }

    #[test]
    fn missing_both_is_an_error() {
        let err = resolve_from(None, None).unwrap_err();
        assert_eq!(err, Error::NoHome);
    }

    #[test]
    fn empty_override_with_no_home_is_an_error() {
        let err = resolve_from(Some(OsStr::new("")), None).unwrap_err();
        assert_eq!(err, Error::NoHome);
    }

    #[test]
    fn models_path_appends_filename() {
        assert_eq!(
            models_path(Path::new("/opt/lernie")),
            PathBuf::from("/opt/lernie/models.yaml")
        );
    }

    #[test]
    fn live_resolve_returns_some_path() {
        // The live resolver must produce *something* on this host: the
        // test harness has either LERNIE_HOME set or a home directory.
        // Asserting only that it succeeds keeps the test independent of
        // the runner's environment.
        let _ = resolve().expect("either LERNIE_HOME or HOME must be set");
    }

    #[test]
    fn override_value_with_path_separator_is_preserved() {
        let v = OsString::from("/srv/data/lernie");
        let p = resolve_from(Some(v.as_os_str()), None).unwrap();
        assert_eq!(p, PathBuf::from("/srv/data/lernie"));
    }
}
