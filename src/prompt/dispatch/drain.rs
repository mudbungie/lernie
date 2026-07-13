//! Step-boundary inbox drain (ARCH §2.11 *Delivery*).
//!
//! At each step boundary — after the prior step's tool entries commit,
//! before the next model call's context assembles (§2.3 drain ordering) —
//! the executor drains the agent's inbox. [`drain`] first commits any
//! renamed-but-uncommitted stray a prior death left in `messages/` (the
//! file left the inbox in a rename that never reached its commit, so it
//! must land — the inverse of §2.3's "an entry that never committed never
//! happened"), then moves each pending inbox message into
//! `messages/NNN-<sender>.md` and commits it (the delivery commit,
//! [`transcript::deliver_message`]).
//!
//! Arrival order across senders is advisory (mtimes); the committed
//! sequence is the record (§2.11). The drain order is deterministic —
//! `(mtime, filename)` — so the same inbox always yields the same
//! committed sequence and replay agrees with the live run.

use super::{transcript, transfer};
use crate::prompt::Error;
use crate::template::GitRunner;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Extension of a deposited inbox message file (§2.11 *Deposit* —
/// `<sender>-<NNN>.md`).
const MESSAGE_EXT: &str = "md";

/// Drain `inbox` into the branch `worktree` at a step boundary (§2.11).
/// The whole drain runs inside the executor lock the step loop already
/// holds, so [`transcript::next_seq`]'s max-present-plus-one counter is
/// race-free (§2.3).
pub(super) fn drain(
    worktree: &Path,
    inbox: &Path,
    conv_id: &str,
    git: &dyn GitRunner,
) -> Result<(), Error> {
    recover_strays(worktree, conv_id, git)?;
    for msg in pending(inbox)? {
        // A result message (§2.6) carries a `terminal_ref:` and applies
        // its work-product transfer as one commit *before* its own
        // delivery commit (§2.6, §2.11). An ordinary steering message
        // carries none and delivers directly.
        let body = std::fs::read_to_string(&msg.path).map_err(Error::Io)?;
        if let Some(terminal_ref) = transfer::terminal_ref_of(&body) {
            transfer::apply(worktree, &msg.sender, &terminal_ref, git)?;
        }
        transcript::deliver_message(worktree, conv_id, &msg.sender, &msg.path, git)?;
    }
    Ok(())
}

/// One deliverable inbox message, carrying the sort keys `(mtime, name)`
/// (§2.11 — advisory arrival order made a deterministic committed
/// sequence).
#[derive(Debug)]
pub(super) struct Pending {
    mtime: SystemTime,
    name: String,
    path: PathBuf,
    sender: String,
}

/// List the inbox's deliverable messages, sorted `(mtime, filename)`. An
/// absent inbox yields nothing — the general path with empty inputs, not
/// a bootstrap special case. A name that is not a well-formed
/// `<sender>-<NNN>.md` deposit (an in-flight `.tmp`, a stray) contributes
/// nothing: only real deposits are delivered. `pub(super)` for the
/// launched driver's found-anything test ([`super::driver`]).
pub(super) fn pending(inbox: &Path) -> Result<Vec<Pending>, Error> {
    let rd = match std::fs::read_dir(inbox) {
        Ok(rd) => rd,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(Error::Io(e)),
    };
    let mut out = Vec::new();
    for entry in rd {
        let entry = entry.map_err(Error::Io)?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(sender) = sender_of(&name) else {
            continue;
        };
        // mtime is advisory — the committed counter is the record
        // (§2.11). An unreadable one sorts to the epoch (delivered first);
        // any genuine trouble with the file resurfaces at its rename.
        let mtime = entry
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        out.push(Pending {
            mtime,
            name,
            path: entry.path(),
            sender,
        });
    }
    out.sort_by(|a, b| (a.mtime, &a.name).cmp(&(b.mtime, &b.name)));
    Ok(out)
}

/// The `<sender>` of a `<sender>-<NNN>.md` deposit name (§2.11 *Deposit*):
/// strip the `.md` suffix, split off the trailing `-<NNN>` numeric
/// sequence, and keep the remainder — so `user-001.md` → `user` and a
/// hyphenated agent id `a-b-c-002.md` → `a-b-c`. A name with no numeric
/// tail, the wrong extension, an empty sender, or a leading-dot temp
/// prefix yields `None`.
fn sender_of(name: &str) -> Option<String> {
    let stem = name.strip_suffix(&format!(".{MESSAGE_EXT}"))?;
    let (sender, seq) = stem.rsplit_once('-')?;
    seq.parse::<u32>().ok()?;
    (!sender.is_empty() && !sender.starts_with('.')).then(|| sender.to_string())
}

/// Commit any renamed-but-uncommitted stray a prior death left under
/// `messages/` (§2.11): a delivery whose `rename(2)` landed but whose
/// commit never ran leaves the file untracked in the worktree, and it
/// must land. Scoped to `messages/` so a worktree left dirty elsewhere (a
/// tool side effect mid-step) is never swept into the delivery commit. A
/// clean `messages/` — the common case — reports nothing and this is a
/// no-op.
fn recover_strays(worktree: &Path, conv_id: &str, git: &dyn GitRunner) -> Result<(), Error> {
    let status = git
        .run_capture(
            worktree,
            &["status", "--porcelain", "--", transcript::MESSAGES_DIR],
        )
        .map_err(|source| Error::Git {
            op: "drain status",
            source,
        })?;
    if status.trim().is_empty() {
        return Ok(());
    }
    git.run(worktree, &["add", transcript::MESSAGES_DIR])
        .map_err(|source| Error::Git {
            op: "drain recover add",
            source,
        })?;
    let msg = format!("transcript: recover delivered stray [{conv_id}]");
    git.run(worktree, &["commit", "-m", msg.as_str()])
        .map_err(|source| Error::Git {
            op: "drain recover commit",
            source,
        })
}

#[cfg(test)]
mod tests;
