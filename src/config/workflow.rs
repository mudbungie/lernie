//! `<conv-repo>/workflow.yaml` — event-to-action bindings per ARCH §6.
//!
//! Workflows are declarative: events are drawn from a closed set, actions
//! are drawn from another closed set (defined in [`crate::config::action`]).
//! Adding either is intentionally a code change.

use crate::config::action::Action;
use crate::config::error::LoadError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Top-level `workflow.yaml` shape.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct Workflow {
    pub events: BTreeMap<Event, Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compaction: Option<CompactionConfig>,
    /// Retry policy for a step's model call (ARCH §2.10, §4.4): the
    /// harness owns the retry loop (brazen never retries), reading the
    /// attempt cap and backoff from here. Omitted uses [`RetryConfig::
    /// default`].
    #[serde(default)]
    pub retry: RetryConfig,
}

/// Harness-owned retry policy (ARCH §6 `retry:` block). One `bz`
/// process per attempt (§4.4); a retryable in-band `Error` re-invokes
/// `bz` with the identical request up to `max_attempts`, sleeping the
/// backoff between attempts (§2.10).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RetryConfig {
    /// Attempt cap per model call — `1` disables retry (a single
    /// attempt). Each attempt is one `bz` invocation appending one
    /// segment to `response.json` (§4.4).
    pub max_attempts: u32,
    pub backoff: Backoff,
}

impl Default for RetryConfig {
    fn default() -> Self {
        // Matches the ARCH §6 example: 3 attempts, exponential backoff.
        Self {
            max_attempts: 3,
            backoff: Backoff::Exponential,
        }
    }
}

/// Closed set of backoff policies between retry attempts (ARCH §6).
/// Exponential is the shipped policy; adding another is a code change,
/// like every other closed workflow set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Backoff {
    Exponential,
}

/// Base delay before the first retry (the exponential's first rung).
const BACKOFF_BASE_MS: u64 = 250;

impl Backoff {
    /// Delay to sleep before the retry that follows a failed `attempt`
    /// (1-based). Exponential doubles per rung from [`BACKOFF_BASE_MS`],
    /// saturating so a pathological attempt count cannot overflow.
    pub fn delay(self, attempt: u32) -> std::time::Duration {
        match self {
            Backoff::Exponential => {
                let factor = 2u64.saturating_pow(attempt.saturating_sub(1));
                std::time::Duration::from_millis(BACKOFF_BASE_MS.saturating_mul(factor))
            }
        }
    }
}

/// Closed set of workflow events. Names match the arch examples.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    UserMessage,
    WorkerReturn,
    VerifierApprove,
    VerifierReject,
    WorkerFlush,
    BranchStopped,
    PreStep,
    PostStep,
    OnToolReturn,
}

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
}

/// Closed set of intermediate-compaction triggers (ARCH §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CompactionTrigger {
    EveryNCommits,
    EveryTSeconds,
    OnFlush,
}

impl Workflow {
    /// Read, parse, and validate `workflow.yaml` at `path`. Each action
    /// string is parsed against the closed action set.
    pub fn load(path: &Path) -> Result<Self, LoadError> {
        let raw = fs::read_to_string(path).map_err(|source| LoadError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let parsed: Self = serde_yaml_ng::from_str(&raw).map_err(|source| LoadError::Yaml {
            path: path.to_path_buf(),
            source,
        })?;
        parsed.validate(path)?;
        Ok(parsed)
    }

    fn validate(&self, path: &Path) -> Result<(), LoadError> {
        for (event, actions) in &self.events {
            for (i, raw) in actions.iter().enumerate() {
                Action::parse(raw).map_err(|message| LoadError::Invalid {
                    path: path.to_path_buf(),
                    key: format!("events.{}[{i}]", event_name(*event)),
                    message,
                })?;
            }
        }
        if let Some(compaction) = &self.compaction {
            validate_compaction(path, compaction)?;
        }
        Ok(())
    }

    /// Pre-parse every action string into a typed [`Action`]. Intended for
    /// callers that want the typed view without re-validating.
    pub fn typed_events(&self) -> BTreeMap<Event, Vec<Action>> {
        self.events
            .iter()
            .map(|(event, actions)| {
                let parsed = actions
                    .iter()
                    .map(|raw| Action::parse(raw).expect("validated at load"))
                    .collect();
                (*event, parsed)
            })
            .collect()
    }
}

fn validate_compaction(path: &Path, c: &CompactionConfig) -> Result<(), LoadError> {
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
    Ok(())
}

fn event_name(event: Event) -> &'static str {
    match event {
        Event::UserMessage => "user_message",
        Event::WorkerReturn => "worker_return",
        Event::VerifierApprove => "verifier_approve",
        Event::VerifierReject => "verifier_reject",
        Event::WorkerFlush => "worker_flush",
        Event::BranchStopped => "branch_stopped",
        Event::PreStep => "pre_step",
        Event::PostStep => "post_step",
        Event::OnToolReturn => "on_tool_return",
    }
}

// Tests for `workflow.yaml` parsing live in `tests/workflow_yaml.rs` so this
// file stays under the 300-line code-file limit.
