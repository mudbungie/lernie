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
    /// Per-conversation spend limits (ARCH §6 "Budgets (v0.7)"). Checked
    /// at every model-call boundary before invoking the adapter; every
    /// value is derived at check time from on-disk `Usage` events, step
    /// timestamps, and branch depth — the harness stores no running
    /// counter (PRINCIPLES "Single source of truth"). Omitted → every
    /// limit unbounded.
    #[serde(default)]
    pub budgets: Budgets,
}

/// Per-conversation spend limits (ARCH §6 `budgets:` block, v0.7). Each
/// limit is optional; an omitted limit is unbounded. All three are derived
/// live from disk at check time — never stored — and are a whole-tree
/// ceiling: any driver (root or subagent) checks the tree's total spend
/// against the single frozen limit, with no per-dispatch inheritance
/// (ARCH §6; the tree shares one `steps/` per §2.2/§2.3).
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct Budgets {
    /// Cap on tokens summed across *every* attempt segment of every
    /// step's `response.json` in the conversation tree — failed and
    /// superseded attempts are billed too (ARCH §6/§8; the
    /// last-segment-authoritative rule governs *context*, not billing).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_total_tokens: Option<u64>,
    /// Cap on wall-clock seconds summed per step from `meta.json`'s
    /// `started_at`→`ended_at`; each span already includes the backoff
    /// sleeps between that step's attempts (ARCH §6 "wall is wall",
    /// §2.10).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_wall_seconds: Option<u64>,
    /// Cap on the conversation's dispatch depth (root = 0; each dispatch
    /// is one deeper). A conversation deeper than this exhausts on its
    /// first model call (ARCH §6).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<u32>,
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
    CompactorReturn,
    BranchStopped,
    PreStep,
    PostStep,
    OnToolReturn,
}

impl Event {
    /// The `workflow.yaml` key for this event (ARCH §6) — the stable name
    /// used in diagnostics and by the runtime interpreter.
    pub fn as_str(self) -> &'static str {
        event_name(self)
    }
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
        Self::parse(&raw, path)
    }

    /// Parse and validate workflow YAML already in hand — the
    /// governing-config read path (ARCH §2.2: control is read from the
    /// config commit's tree, never from a worktree file). `origin`
    /// labels errors (e.g. `<config-commit>:workflow.yaml`).
    pub fn parse(raw: &str, origin: &Path) -> Result<Self, LoadError> {
        let parsed: Self = serde_yaml_ng::from_str(raw).map_err(|source| LoadError::Yaml {
            path: origin.to_path_buf(),
            source,
        })?;
        parsed.validate(origin)?;
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

    /// The typed actions bound to one `event`, in declared order (ARCH §6
    /// "The binding interpreter" — the flat list the hop matches against
    /// disk circumstance). An unbound event yields the empty list: the
    /// general path with empty inputs, not a bootstrap special case.
    /// Strings were validated at load, so parsing here cannot fail.
    pub fn actions_for(&self, event: Event) -> Vec<Action> {
        self.events
            .get(&event)
            .map(|raw| {
                raw.iter()
                    .map(|s| Action::parse(s).expect("validated at load"))
                    .collect()
            })
            .unwrap_or_default()
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
        Event::CompactorReturn => "compactor_return",
        Event::BranchStopped => "branch_stopped",
        Event::PreStep => "pre_step",
        Event::PostStep => "post_step",
        Event::OnToolReturn => "on_tool_return",
    }
}

// Tests for `workflow.yaml` parsing live in `tests/workflow_yaml.rs` so this
// file stays under the 300-line code-file limit.
