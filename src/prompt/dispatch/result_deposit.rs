//! Result-message deposit at a terminal event (ARCH §2.6, §2.3 step 5).
//!
//! Every terminal event of a step loop deposits a **result message** —
//! "Return is not a verb" (`docs/PRINCIPLES.md`): the deposit is
//! executor-side, never a model `message` tool call. This module is the
//! executor's side of that return: it derives the recipient
//! ([`recipient`]), reads the branch tip as the terminal ref (§2.6), and
//! deposits with the matching epitaph.
//!
//! **Who the result is addressed to is decided by the epitaph's value**
//! (§2.6 — code branches on the value, never on the message's shape):
//! a **reply** (`final-response`) answers whoever last prompted this
//! agent; an **obituary** (`stopped`, `budget-exhausted`, `died`) reports
//! to the dispatcher, whose address is the agent's own id minus its last
//! descent segment ([`inbox::parent_of`], §2.11). Both can be absent —
//! a reply to the user and a root's obituary alike deposit nothing — and
//! the absent arm is one structural no-op, not two special cases (§2.4:
//! the terminal response answers the user, who reads this agent's own
//! conversation).
//!
//! **The deposit does not launch.** Waking the recipient this deposit
//! revives (§2.11 revival-on-deposit) is the exit protocol's closing
//! act, not the deposit's return value: it happens once, after the
//! depositing executor releases its own lock, and by epitaph value —
//! [`super::terminal::exit_launch`] / `revive_recipient`, which addresses
//! the same [`recipient`] the deposit did. Keeping it there keeps this a
//! pure return and keeps the launch decision in one place.

use super::super::inbox::{self, Epitaph, USER_SENDER};
use super::super::{Deps, Error};
use super::step_commit::read_branch_tip;
use super::transcript::MESSAGES_DIR;
use super::transfer::terminal_ref_of;
use brazen::Content;
use std::path::Path;

/// Extension of a delivered message's transcript entry (§2.3 —
/// `messages/NNN-<sender>.md`; model output and tool results are
/// `.json`, so the extension alone separates speech from step output).
const DELIVERED_EXT: &str = ".md";

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

/// The inbox this terminal event's result message is addressed to (§2.6
/// *A reply answers the last prompter; an obituary reports to the
/// dispatcher*), or `None` when no agent is addressed.
///
/// An **obituary** — every epitaph but `final-response` — is a
/// structural fact about the tree rather than an answer to anyone: it
/// says *this agent is gone*, and the one party that has a standing
/// interest in that is the agent that dispatched it. Its address is the
/// id's ([`inbox::parent_of`]), which no rewrite of the transcript can
/// move, so the dispatcher hears about a stop, an exhausted ceiling or a
/// death even when the branch was mid-conversation with somebody else.
///
/// A **reply** — `final-response` — answers whoever last prompted this
/// agent ([`last_prompter`]). For the dispatch step the last prompter
/// *is* the dispatcher (the goal arrives as its message, §2.5), so the
/// old parent-addressed rule is this rule's first case rather than a
/// rule of its own; `user` is nobody's inbox, so an operator-prompted
/// reply deposits nothing and is read in this agent's own conversation.
pub(super) fn recipient(
    worktree: &Path,
    agent_id: &str,
    epitaph: Epitaph,
) -> Result<Option<String>, Error> {
    if epitaph != Epitaph::FinalResponse {
        return Ok(inbox::parent_of(agent_id));
    }
    Ok(match last_prompter(worktree, agent_id)? {
        Some(sender) if sender == USER_SENDER => None,
        Some(sender) => Some(sender),
        // No surviving prompt: the dispatch message is the transcript's
        // first, so its absence means compaction squashed the record
        // (§2.6) — and the id still carries the one sender the branch's
        // own existence records. A root has neither, and deposits
        // nothing.
        None => inbox::parent_of(agent_id),
    })
}

/// The **last prompter**: the sender of the newest delivered message in
/// this branch's transcript that is a prompt from somebody else (§2.3 —
/// order lives in the filename, so the newest is max-`NNN`; the origin
/// token names the sender). Derived, never stored: a stored "who spoke
/// last" would be a second copy of what `messages/` already says
/// (`docs/PRINCIPLES.md` Single source of truth).
///
/// Two entries are skipped, each because it is not a prompt:
///
/// - **A returning child's result message** — a delivered entry carrying
///   `terminal_ref:` frontmatter (§2.6). It is the answer to a dispatch
///   this agent already made, not a question put to it; without the skip
///   every parent would address its own answer to the last child that
///   returned.
/// - **This agent's own note to itself** (§2.11 *Self-messages*). Its
///   answer is the agent's own next step, which has already happened;
///   addressing a reply to one's own inbox would deposit into the very
///   inbox whose delivery produced it and never terminate.
fn last_prompter(worktree: &Path, agent_id: &str) -> Result<Option<String>, Error> {
    let dir = worktree.join(MESSAGES_DIR);
    let rd = match std::fs::read_dir(&dir) {
        Ok(rd) => rd,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(Error::Io(e)),
    };
    let mut delivered: Vec<(u32, String, std::path::PathBuf)> = Vec::new();
    for entry in rd {
        let entry = entry.map_err(Error::Io)?;
        let name = entry.file_name();
        let Some((seq, sender)) = name.to_str().and_then(parse_delivered) else {
            continue;
        };
        if sender != agent_id {
            delivered.push((seq, sender.to_string(), entry.path()));
        }
    }
    delivered.sort_unstable();
    for (_, sender, path) in delivered.into_iter().rev() {
        if terminal_ref_of(&std::fs::read_to_string(path)?).is_none() {
            return Ok(Some(sender));
        }
    }
    Ok(None)
}

/// Split `NNN-<sender>.md` into its counter and its origin token (§2.3).
/// A sender id carries hyphens of its own (the descent, §2.3), so the
/// split is at the *first* hyphen and the remainder is the whole token.
/// Anything else under `messages/` — a `.json` step entry, a stray — is
/// `None`.
fn parse_delivered(name: &str) -> Option<(u32, &str)> {
    let (seq, sender) = name.strip_suffix(DELIVERED_EXT)?.split_once('-')?;
    let seq = seq.parse().ok()?;
    (!sender.is_empty()).then_some((seq, sender))
}

/// Deposit this branch's result message on its own behalf at a terminal
/// event (§2.3 step 5): derive the recipient ([`recipient`]), read the
/// branch tip as the terminal ref, then deposit with `epitaph` and the
/// `response` body. Addressed to nobody — an operator-prompted reply, a
/// root's obituary — it is one structural no-op; wired at the call site
/// so a step loop returns without new plumbing.
pub(super) fn deposit_terminal(
    repo: &Path,
    conv_id: &str,
    worktree: &Path,
    epitaph: Epitaph,
    response: Option<&str>,
    deps: &Deps<'_>,
) -> Result<(), Error> {
    let Some(recipient) = recipient(worktree, conv_id, epitaph)? else {
        return Ok(());
    };
    let terminal_ref = read_branch_tip(worktree, deps)?;
    inbox::deposit_result(
        repo,
        &recipient,
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
mod tests;
