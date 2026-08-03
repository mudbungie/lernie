//! The `compaction:` block of `workflow.yaml` (ARCH §2.6, §2.7, §6):
//! checkpoint triggers and span selection, split from [`super`] to hold
//! the per-file line cap.

use crate::config::error::LoadError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Optional `compaction:` block (ARCH §6).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CompactionConfig {
    pub intermediate: IntermediateCompaction,
}

/// Configuration for intermediate compaction triggers.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct IntermediateCompaction {
    pub trigger: CompactionTrigger,
    /// Required when `trigger == every_n_commits` (commit count) or
    /// `every_t_seconds` (seconds). Ignored for `on_flush`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    /// Most recent commits kept out of the compaction span (ARCH §2.6):
    /// the compactor forks off `HEAD~keep_recent` — the compaction point
    /// — so the retained tail survives verbatim and replays on top of the
    /// landing. Omitted → `0`, the point is the tip. Must stay below `n`
    /// under `every_n_commits` (validated), else every landing re-arms.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keep_recent: Option<u32>,
}

/// Closed set of intermediate-compaction triggers (ARCH §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CompactionTrigger {
    EveryNCommits,
    EveryTSeconds,
    OnFlush,
}

pub(super) fn validate_compaction(path: &Path, c: &CompactionConfig) -> Result<(), LoadError> {
    let needs_n = matches!(
        c.intermediate.trigger,
        CompactionTrigger::EveryNCommits | CompactionTrigger::EveryTSeconds
    );
    let has_n = c.intermediate.n.is_some_and(|n| n > 0);
    if needs_n && !has_n {
        return Err(LoadError::Invalid {
            path: path.to_path_buf(),
            key: "compaction.intermediate.n".into(),
            message: "must be a positive integer for the chosen trigger".into(),
        });
    }
    let keep = c.intermediate.keep_recent.unwrap_or(0);
    if matches!(c.intermediate.trigger, CompactionTrigger::EveryNCommits)
        && c.intermediate.n.is_some_and(|n| keep >= n)
    {
        return Err(LoadError::Invalid {
            path: path.to_path_buf(),
            key: "compaction.intermediate.keep_recent".into(),
            message: "must be smaller than n: a retained tail at or over the commit \
                      trigger would re-arm the clock at every landing"
                .into(),
        });
    }
    Ok(())
}
