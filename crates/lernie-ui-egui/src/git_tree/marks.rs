//! Ref-derived agent marks (§2.6 declined-transfer, §6 budget-exhausted).
//!
//! Two orthogonal marks, rendered alongside the primary agent state (§3.5,
//! §7.1), each a git ref keyed by agent id:
//!
//! - **declined-transfer** — `refs/lernie/conflicted/<agent-id>`: a
//!   child's work-product transfer failed to apply and was declined loudly
//!   (§2.6).
//! - **budget-exhausted** — `refs/lernie/budget-exhausted/<agent-id>`: the
//!   agent hit a budget ceiling (§6).
//!
//! Derived from `git for-each-ref`, never stored (PRINCIPLES "Single
//! source of truth"). Both ref namespaces key off the raw agent id (no
//! `agents/` prefix), matching the harness's `transfer::decline` and
//! `budget::mark_exhausted`.

use super::GitTreeError;
use super::cmd::for_each_ref_under;
use std::collections::HashSet;
use std::path::Path;

const CONFLICTED_PREFIX: &str = "refs/lernie/conflicted/";
const BUDGET_PREFIX: &str = "refs/lernie/budget-exhausted/";

/// The two mark sets for a workspace, read once per `from_repo` tick.
#[derive(Debug, Default)]
pub(super) struct Marks {
    conflicted: HashSet<String>,
    budget: HashSet<String>,
}

impl Marks {
    pub(super) fn from_repo(git_dir: &Path) -> Result<Self, GitTreeError> {
        Ok(Self {
            conflicted: ids_under(git_dir, CONFLICTED_PREFIX)?,
            budget: ids_under(git_dir, BUDGET_PREFIX)?,
        })
    }

    pub(super) fn declined_transfer(&self, agent_id: &str) -> bool {
        self.conflicted.contains(agent_id)
    }

    pub(super) fn budget_exhausted(&self, agent_id: &str) -> bool {
        self.budget.contains(agent_id)
    }
}

fn ids_under(git_dir: &Path, prefix: &str) -> Result<HashSet<String>, GitTreeError> {
    let out = for_each_ref_under(git_dir, prefix)?;
    Ok(parse_ids(&out, prefix))
}

/// Strip `prefix` off each `refname` line, yielding the agent-id set.
fn parse_ids(stdout: &[u8], prefix: &str) -> HashSet<String> {
    String::from_utf8_lossy(stdout)
        .lines()
        .map(str::trim)
        .filter_map(|r| r.strip_prefix(prefix))
        .filter(|id| !id.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ids_strips_prefix_and_drops_blanks() {
        let out = b"refs/lernie/conflicted/a-b\nrefs/lernie/conflicted/c-d\n";
        let ids = parse_ids(out, CONFLICTED_PREFIX);
        assert!(ids.contains("a-b"));
        assert!(ids.contains("c-d"));
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn parse_ids_ignores_lines_without_the_prefix() {
        // A stray ref that doesn't match the namespace contributes nothing.
        let out = b"refs/heads/agents/a-b\nrefs/lernie/budget-exhausted/x-y\n";
        let ids = parse_ids(out, BUDGET_PREFIX);
        assert_eq!(ids.len(), 1);
        assert!(ids.contains("x-y"));
    }

    #[test]
    fn parse_ids_empty_input_is_empty_set() {
        assert!(parse_ids(b"", CONFLICTED_PREFIX).is_empty());
    }

    #[test]
    fn lookups_reflect_membership() {
        let marks = Marks {
            conflicted: HashSet::from(["a-b".to_string()]),
            budget: HashSet::from(["c-d".to_string()]),
        };
        assert!(marks.declined_transfer("a-b"));
        assert!(!marks.declined_transfer("c-d"));
        assert!(marks.budget_exhausted("c-d"));
        assert!(!marks.budget_exhausted("a-b"));
    }
}
