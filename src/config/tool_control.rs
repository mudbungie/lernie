//! `workflow.yaml`'s `tool_control:` block — the **tool control** seam
//! (ARCH §3.3 *Tool control*).
//!
//! A tool control is an operator-configured adjudicator consulted before
//! every granted tool invocation executes: it answers **pass**, **refuse**
//! or **hold** (`crate::prompt::tool::control`). This block is only the
//! seam's config home — it names the control binary and nothing else;
//! all policy (which tools, which roles, who releases a hold) lives in
//! the control itself, read from the governing config commit like every
//! other workflow fact (§2.2).
//!
//! Like `tool_output:`, the block is severable policy: omitting it means
//! no control is consulted and the tool window behaves exactly as it did
//! before the seam existed — the general path with the policy absent.
//! No control ships in `template/workflow.yaml`; wiring one is config,
//! removing one deletes config, never code (PRINCIPLES, severability).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The `tool_control:` block: the control binary the seam consults.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ToolControl {
    /// The control executable — an absolute path or a name resolved on
    /// `PATH` at spawn (the §4.2 `adapter:` override idiom: config names
    /// a binary, the OS resolves it). Invoked with no arguments; the
    /// invocation under adjudication arrives as JSON on stdin (§3.3
    /// *Tool control*). Must be non-empty (validated at load).
    pub command: String,
}
