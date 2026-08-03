//! Result-message deposit at a terminal event (ARCH §2.6, §2.3 step 5).
//!
//! Every terminal event of a step loop deposits a **result message**
//! into the parent's inbox — "Return is not a verb" (`docs/PRINCIPLES.md`):
//! the deposit is executor-side, never a model `message` tool call. This
//! module is the executor's side of that return: it reads the branch tip
//! as the terminal ref (§2.6) and deposits with the matching epitaph.
//!
//! For a *root* agent it is a structural no-op — a root has no parent
//! inbox, so [`crate::prompt::inbox::deposit_child_result`] deposits
//! nothing and the root's terminal response answers the user instead
//! (§2.4). Both drivers reach it for real: the root step loop
//! ([`super::run_exchange`]) and a dispatched child's `lernie advance`
//! hop ([`super::advance`], §6).
//!
//! **The deposit does not launch.** Waking the parent this deposit
//! revives (§2.11 revival-on-deposit) is the exit protocol's closing
//! act, not the deposit's return value: it happens once, after the
//! depositing executor releases its own lock, and by epitaph value —
//! [`super::terminal::exit_launch`] / `revive_parent`. Keeping it there
//! keeps this a pure return and keeps the launch decision in one place.

use super::super::inbox::{self, Epitaph};
use super::super::{Deps, Error};
use super::step_commit::read_branch_tip;
use brazen::Content;
use std::path::Path;

/// The terminal response body iff the agent spoke: the concatenated
/// [`Content::Text`] blocks of the final assistant content, or `None`
/// when it produced none (§2.6 — the body is present exactly when the
/// agent spoke). Thinking blocks are not speech and are excluded.
pub(super) fn terminal_text(blocks: &[Content]) -> Option<String> {
    let mut out = String::new();
    for block in blocks {
        if let Content::Text(text) = block {
            out.push_str(text);
        }
    }
    (!out.is_empty()).then_some(out)
}

/// Deposit this branch's result message on its own behalf at a terminal
/// event (§2.3 step 5): read the branch tip as the terminal ref, then
/// deposit into the parent's inbox with `epitaph` and the `response`
/// body. A no-op for a root (no parent inbox, §2.4); wired at the call
/// site so a child step loop returns without new plumbing.
pub(super) fn deposit_terminal(
    repo: &Path,
    conv_id: &str,
    worktree: &Path,
    epitaph: Epitaph,
    response: Option<&str>,
    deps: &Deps<'_>,
) -> Result<(), Error> {
    let terminal_ref = read_branch_tip(worktree, deps)?;
    inbox::deposit_child_result(
        repo,
        conv_id,
        epitaph,
        &terminal_ref,
        response,
        deps.clock,
        deps.git,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn terminal_text_joins_text_blocks_and_skips_non_text() {
        let blocks = vec![
            Content::Text("hello ".into()),
            Content::Thinking {
                text: "ignored".into(),
                signature: None,
                id: None,
                encrypted_content: None,
            },
            Content::ToolUse {
                id: "t1".into(),
                name: "bash".into(),
                input: json!({}),
                signature: None,
            },
            Content::Text("world".into()),
        ];
        assert_eq!(terminal_text(&blocks).as_deref(), Some("hello world"));
    }

    #[test]
    fn terminal_text_is_none_when_agent_did_not_speak() {
        let blocks = vec![Content::ToolUse {
            id: "t1".into(),
            name: "bash".into(),
            input: json!({}),
            signature: None,
        }];
        assert_eq!(terminal_text(&blocks), None);
        assert_eq!(terminal_text(&[]), None);
    }
}
