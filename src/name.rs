//! Boundary validation for outside-supplied names that become path
//! components (ARCH §2.3, §3.3).
//!
//! Two names arrive from outside the harness and are then joined into
//! filesystem paths: an **agent id** — its branch name, its worktree
//! directory, and its `steps/` / `inbox/` namespaces (§2.2, §2.3) — and
//! a **skill name**, which addresses one directory in the data-root
//! pool (§3.3). Both must address exactly one path component, and a
//! name that escapes its base is **declined, never sanitized**
//! (`docs/PRINCIPLES.md` "Decline illegal operations"): the name is a
//! fact, not a slot to munge.
//!
//! The check is load-bearing rather than cosmetic, because `Path::join`
//! is not a containment operation: a `..` segment walks out of the base
//! and an *absolute* name replaces it outright. Each name is validated
//! once, where it enters — the command surface for an agent id (§3.4:
//! every binding, and every model-issued tool, re-enters through a verb),
//! the tool entry for a skill name — so no interior code ever holds a
//! name that cannot be joined safely.

/// True iff `name` addresses exactly one path component: non-empty, not
/// `.` or `..`, and free of `/`, `\`, and NUL.
pub fn is_component(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains('/')
        && !name.contains('\\')
        && !name.contains('\0')
}

/// A declined agent id (ARCH §2.3), rendered inside the verb's uniform
/// `lernie <verb>: <error>` failure line (§3.4).
#[derive(Debug, thiserror::Error)]
#[error(
    "agent id {0:?} is not a single path component — an agent id is its branch name, \
     the hyphenated descent `<a>-<b>-…` (ARCH §2.3); pass the id exactly as \
     `lernie prompt` / `lernie dispatch` printed it"
)]
pub struct NotAnAgentId(String);

/// Decline an agent id that is not a single path component — the guard
/// every verb that takes an id from outside runs before touching disk.
pub fn require_agent_id(id: &str) -> Result<(), NotAnAgentId> {
    if is_component(id) {
        return Ok(());
    }
    Err(NotAnAgentId(id.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::{is_component, require_agent_id};

    #[test]
    fn a_plain_name_is_one_component() {
        assert!(is_component("20260101-a1"));
        assert!(require_agent_id("20260101-a1-20260102-b2").is_ok());
    }

    #[test]
    fn escapes_and_empties_are_declined() {
        for bad in [
            "",
            ".",
            "..",
            "../../victim/pwned",
            "/etc/lernie",
            "a\\b",
            "a\0b",
        ] {
            assert!(!is_component(bad), "{bad:?} must not pass");
            assert!(require_agent_id(bad).is_err(), "{bad:?} must be declined");
        }
    }

    #[test]
    fn the_decline_names_the_id_and_the_rule() {
        let err = require_agent_id("../../victim/pwned").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("\"../../victim/pwned\""), "{msg}");
        assert!(msg.contains("single path component"), "{msg}");
        assert!(msg.contains("§2.3"), "{msg}");
    }
}
