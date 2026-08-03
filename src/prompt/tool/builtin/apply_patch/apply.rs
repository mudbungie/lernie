//! Application of a parsed patch (ARCH §3.3 *The patch tool*).
//!
//! Two phases, which is where atomicity lives: **stage** reads every
//! target and computes every post-state in memory — every decline
//! (missing file, existing add target, context not found or ambiguous)
//! fires here, before any byte on disk has changed — then **write**
//! lands the staged states. A patch that cannot apply in full applies
//! not at all. (A write-phase I/O fault can still stop midway; it is
//! surfaced verbatim, and the per-invocation tool commit records the
//! exact resulting tree either way.)
//!
//! The stale-state guard (bl-e249) is structural, not a version number:
//! an add declines when the path already exists, an update or delete
//! declines when it does not, and an update's hunks must locate their
//! authored context through the [`super::seek`] matching ladder —
//! uniquely. Content that drifted since the model last read it stops
//! matching and the patch is refused with the exact reason, never
//! applied over unseen changes.

use super::parse::{FileOp, Hunk, Patch};
use super::report::{FileReport, HunkReport, Report, entry};
use super::seek::{self, LADDER};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Why the patch was refused. Every variant names the file, the hunk
/// where applicable, and the exact reason — the model repairs from this
/// text alone.
#[derive(Debug, Error)]
pub enum Error {
    /// The add target already exists: the patch was authored against a
    /// tree that did not have it, so applying would overwrite unseen
    /// content. Update it or delete it first.
    #[error("add {path}: file already exists; update it or delete it first")]
    AddExists { path: String },
    /// Reading a target failed — missing file (the stale-state case for
    /// update/delete), permission, or non-UTF-8 content (the tool edits
    /// text; binary files stay `bash`'s business).
    #[error("{action} {path}: {source}")]
    Io {
        action: &'static str,
        path: String,
        #[source]
        source: io::Error,
    },
    /// The rename target of a `*** Move to:` already exists.
    #[error("move {path} to {to}: destination already exists")]
    MoveDestExists { path: String, to: String },
    /// A hunk's anchor or context block was not found from the cursor
    /// on, at any rung of the matching ladder.
    #[error("update {path}, hunk {hunk}: {what} {source}")]
    NotFound {
        path: String,
        hunk: usize,
        what: String,
        source: seek::Error,
    },
    /// The first rung to match at all matched more than once: a loud
    /// decline, never a guessed edit. `@@ <enclosing symbol>` anchor
    /// lines or more context make the target unique.
    #[error(
        "update {path}, hunk {hunk}: {what} is ambiguous — {source}; \
         add an `@@ <enclosing symbol>` anchor line or more context"
    )]
    Ambiguous {
        path: String,
        hunk: usize,
        what: String,
        source: seek::Error,
    },
    /// A pure-insertion hunk (no context, no removals) with neither an
    /// `@@` anchor nor `*** End of File` has no defined landing point.
    #[error(
        "update {path}, hunk {hunk}: insertion has no location; give it \
         context lines, an `@@` anchor, or `*** End of File`"
    )]
    UnanchoredInsertion { path: String, hunk: usize },
}

/// A staged operation: the fully computed post-state, ready to write.
enum Staged {
    Add {
        abs: PathBuf,
        report: FileReport,
        content: String,
    },
    Delete {
        abs: PathBuf,
        report: FileReport,
    },
    Update {
        abs: PathBuf,
        move_abs: Option<PathBuf>,
        report: FileReport,
        content: String,
    },
}

/// Apply `patch` with paths resolved against `root` (the calling
/// agent's current working directory; an absolute patch path stands as
/// itself, `Path::join` semantics). All-or-nothing per the module doc.
pub fn apply(patch: &Patch, root: &Path) -> Result<Report, Error> {
    let staged: Vec<Staged> = patch
        .ops
        .iter()
        .map(|op| stage(op, root))
        .collect::<Result<_, _>>()?;
    let files = staged.into_iter().map(write).collect::<Result<_, _>>()?;
    Ok(Report {
        status: "applied",
        files,
    })
}

/// Phase one: validate one operation and compute its post-state.
fn stage(op: &FileOp, root: &Path) -> Result<Staged, Error> {
    match op {
        FileOp::Add { path, lines } => {
            let abs = root.join(path);
            if abs.exists() {
                return Err(Error::AddExists { path: path.clone() });
            }
            let content = if lines.is_empty() {
                String::new()
            } else {
                lines.join("\n") + "\n"
            };
            Ok(Staged::Add {
                abs,
                report: entry(path, "add", None, Vec::new()),
                content,
            })
        }
        FileOp::Delete { path } => {
            let abs = root.join(path);
            // The read is the existence guard — and preserves the
            // pre-state in the staged view before removal.
            read(&abs, path)?;
            Ok(Staged::Delete {
                abs,
                report: entry(path, "delete", None, Vec::new()),
            })
        }
        FileOp::Update {
            path,
            move_to,
            hunks,
        } => {
            let abs = root.join(path);
            let text = read(&abs, path)?;
            let move_abs = match move_to {
                Some(to) => {
                    let dest = root.join(to);
                    if dest.exists() {
                        return Err(Error::MoveDestExists {
                            path: path.clone(),
                            to: to.clone(),
                        });
                    }
                    Some(dest)
                }
                None => None,
            };
            let (content, applied) = run_hunks(&text, hunks, path)?;
            Ok(Staged::Update {
                abs,
                move_abs,
                report: entry(path, "update", move_to.clone(), applied),
                content,
            })
        }
    }
}

fn read(abs: &Path, path: &str) -> Result<String, Error> {
    fs::read_to_string(abs).map_err(|source| Error::Io {
        action: "read",
        path: path.to_string(),
        source,
    })
}

fn io_err(action: &'static str, abs: &Path) -> impl FnOnce(io::Error) -> Error {
    let path = abs.display().to_string();
    move |source| Error::Io {
        action,
        path,
        source,
    }
}

/// Locate and splice every hunk, in order, cursor moving forward.
fn run_hunks(text: &str, hunks: &[Hunk], path: &str) -> Result<(String, Vec<HunkReport>), Error> {
    let had_nl = text.ends_with('\n');
    let mut lines: Vec<String> = if text.is_empty() {
        Vec::new()
    } else {
        text.split('\n').map(String::from).collect()
    };
    if had_nl {
        lines.pop();
    }
    let mut cursor = 0usize;
    let mut applied = Vec::new();
    for (n, hunk) in hunks.iter().enumerate() {
        let hunk_no = n + 1;
        for anchor in &hunk.anchors {
            let what = format!("anchor {anchor:?}");
            let (pos, _) = seek::seek(&lines, std::slice::from_ref(anchor), cursor, false)
                .map_err(located(path, hunk_no, what))?;
            cursor = pos + 1;
        }
        if hunk.old.is_empty() && !hunk.eof && hunk.anchors.is_empty() {
            return Err(Error::UnanchoredInsertion {
                path: path.to_string(),
                hunk: hunk_no,
            });
        }
        // An empty needle (pure insertion) always locates — at the end
        // under `eof`, else right after the anchor — so the miss arm is
        // only reachable with context lines present.
        let (pos, rung) = seek::seek(&lines, &hunk.old, cursor, hunk.eof).map_err(located(
            path,
            hunk_no,
            "context".to_string(),
        ))?;
        let matched: Vec<String> = lines[pos..pos + hunk.old.len()].to_vec();
        lines.splice(pos..pos + hunk.old.len(), hunk.new.iter().cloned());
        cursor = pos + hunk.new.len();
        applied.push(HunkReport {
            rung: rung.label(),
            line: pos + 1,
            matched: (rung != LADDER[0]).then_some(matched),
        });
    }
    let mut content = lines.join("\n");
    if had_nl && !content.is_empty() {
        content.push('\n');
    }
    Ok((content, applied))
}

/// Convert a seek miss into the right decline, with its location facts.
fn located(path: &str, hunk: usize, what: String) -> impl FnOnce(seek::Error) -> Error {
    let path = path.to_string();
    move |source| match source {
        seek::Error::NotFound => Error::NotFound {
            path,
            hunk,
            what,
            source,
        },
        seek::Error::Ambiguous { .. } => Error::Ambiguous {
            path,
            hunk,
            what,
            source,
        },
    }
}

/// Phase two: land one staged operation on disk.
fn write(staged: Staged) -> Result<FileReport, Error> {
    match staged {
        Staged::Add {
            abs,
            report,
            content,
        } => {
            if let Some(parent) = abs.parent().filter(|p| !p.as_os_str().is_empty()) {
                fs::create_dir_all(parent).map_err(io_err("create directory for", &abs))?;
            }
            fs::write(&abs, content).map_err(io_err("write", &abs))?;
            Ok(report)
        }
        Staged::Delete { abs, report } => {
            fs::remove_file(&abs).map_err(io_err("delete", &abs))?;
            Ok(report)
        }
        Staged::Update {
            abs,
            move_abs,
            report,
            content,
        } => {
            fs::write(&abs, content).map_err(io_err("write", &abs))?;
            if let Some(dest) = move_abs {
                if let Some(parent) = dest.parent().filter(|p| !p.as_os_str().is_empty()) {
                    fs::create_dir_all(parent).map_err(io_err("create directory for", &dest))?;
                }
                fs::rename(&abs, &dest).map_err(io_err("rename", &abs))?;
            }
            Ok(report)
        }
    }
}
