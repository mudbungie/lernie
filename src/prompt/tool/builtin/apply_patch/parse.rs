//! Grammar for the `apply_patch` envelope (ARCH §3.3 *The patch tool*).
//!
//! The grammar is codex's `apply_patch` — chosen because models are
//! trained on it, not because it is elegant to parse (the ball's own
//! criterion). One envelope opens with `*** Begin Patch`, closes with
//! `*** End Patch`, and carries any number of file sections:
//!
//! - `*** Add File: <path>` — every following `+`-prefixed line is file
//!   content.
//! - `*** Delete File: <path>` — one line, no body.
//! - `*** Update File: <path>` — optionally `*** Move to: <path>` on the
//!   next line, then one or more hunks: `@@` separates hunks, `@@ <text>`
//!   names an anchor line to locate first (the "@@ enclosing symbol"
//!   disambiguation), ` `-prefixed lines are context, `-` removals, `+`
//!   additions, and `*** End of File` pins the hunk to the file's end.
//!
//! A bare empty line inside a body is read as an empty context line —
//! models routinely drop the lone space on blank lines. Everything else
//! unrecognized is a typed decline naming the line, never a guess.

use thiserror::Error;

const BEGIN: &str = "*** Begin Patch";
const END: &str = "*** End Patch";
const ADD: &str = "*** Add File: ";
const DELETE: &str = "*** Delete File: ";
const UPDATE: &str = "*** Update File: ";
const MOVE: &str = "*** Move to: ";
const EOF_MARK: &str = "*** End of File";

/// A parsed envelope: the file operations in author order.
#[derive(Debug, PartialEq)]
pub struct Patch {
    pub ops: Vec<FileOp>,
}

/// One file operation. Paths are as authored — resolved against the
/// calling agent's current working directory at apply time.
#[derive(Debug, PartialEq)]
pub enum FileOp {
    Add {
        path: String,
        lines: Vec<String>,
    },
    Delete {
        path: String,
    },
    Update {
        path: String,
        move_to: Option<String>,
        hunks: Vec<Hunk>,
    },
}

/// One located edit inside an update: anchors are sought first (each
/// moves the cursor past its match), then `old` is located as a block
/// and replaced by `new`. Context lines appear in both sequences.
#[derive(Debug, Default, PartialEq)]
pub struct Hunk {
    pub anchors: Vec<String>,
    pub old: Vec<String>,
    pub new: Vec<String>,
    pub eof: bool,
}

impl Hunk {
    fn is_blank(&self) -> bool {
        self.anchors.is_empty() && self.old.is_empty() && self.new.is_empty() && !self.eof
    }
}

/// Why the envelope did not parse. Every variant names the offense
/// precisely — the model reads this verbatim and repairs the patch.
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("patch must start with {BEGIN:?}")]
    MissingBegin,
    #[error("patch must end with {END:?}")]
    MissingEnd,
    #[error("patch contains no file operations")]
    Empty,
    #[error(
        "line {line}: unrecognized patch line {content:?}; inside an update, \
         lines start with ' ' (context), '-' (removal), '+' (addition), or '@@'"
    )]
    BadLine { line: usize, content: String },
    #[error("line {line}: {MOVE:?} must directly follow a {UPDATE:?} line")]
    MisplacedMove { line: usize },
    #[error("update of {path} has no hunks")]
    EmptyUpdate { path: String },
    #[error("update of {path}: hunk {hunk} changes nothing")]
    NoChange { path: String, hunk: usize },
    #[error("{path} appears in more than one file operation")]
    DuplicatePath { path: String },
}

/// Parse the envelope text. Leading/trailing blank lines are tolerated;
/// the markers themselves are matched exactly.
pub fn parse(text: &str) -> Result<Patch, Error> {
    let lines: Vec<&str> = text.lines().collect();
    let first = lines.iter().position(|l| !l.trim().is_empty());
    let last = lines.iter().rposition(|l| !l.trim().is_empty());
    let (Some(first), Some(last)) = (first, last) else {
        return Err(Error::MissingBegin);
    };
    if lines[first] != BEGIN {
        return Err(Error::MissingBegin);
    }
    if lines[last] != END {
        return Err(Error::MissingEnd);
    }
    let mut ops = Vec::new();
    let mut i = first + 1;
    while i < last {
        let line = lines[i];
        if let Some(path) = line.strip_prefix(ADD) {
            let (op, next) = parse_add(path, &lines, i + 1, last);
            ops.push(op);
            i = next;
        } else if let Some(path) = line.strip_prefix(DELETE) {
            ops.push(FileOp::Delete {
                path: path.to_string(),
            });
            i += 1;
        } else if let Some(path) = line.strip_prefix(UPDATE) {
            let (op, next) = parse_update(path, &lines, i + 1, last)?;
            ops.push(op);
            i = next;
        } else if line.strip_prefix(MOVE).is_some() {
            return Err(Error::MisplacedMove { line: i + 1 });
        } else if line.trim().is_empty() {
            i += 1;
        } else {
            return Err(Error::BadLine {
                line: i + 1,
                content: line.to_string(),
            });
        }
    }
    if ops.is_empty() {
        return Err(Error::Empty);
    }
    check_duplicates(&ops)?;
    Ok(Patch { ops })
}

/// True when `line` opens a new file section (or is the `Move to` rider).
fn is_section(line: &str) -> bool {
    [ADD, DELETE, UPDATE, MOVE]
        .iter()
        .any(|m| line.starts_with(m))
}

/// Collect an add section's `+`-prefixed content. Returns the op and the
/// index of the first line past the section.
fn parse_add(path: &str, lines: &[&str], from: usize, until: usize) -> (FileOp, usize) {
    let mut content = Vec::new();
    let mut i = from;
    while i < until {
        if let Some(rest) = lines[i].strip_prefix('+') {
            content.push(rest.to_string());
        } else if lines[i].is_empty() {
            content.push(String::new());
        } else {
            break;
        }
        i += 1;
    }
    let op = FileOp::Add {
        path: path.to_string(),
        lines: content,
    };
    (op, i)
}

/// Collect an update section: the optional `Move to` rider, then hunks.
fn parse_update(
    path: &str,
    lines: &[&str],
    from: usize,
    until: usize,
) -> Result<(FileOp, usize), Error> {
    let mut i = from;
    let mut move_to = None;
    if i < until
        && let Some(to) = lines[i].strip_prefix(MOVE)
    {
        move_to = Some(to.to_string());
        i += 1;
    }
    let mut hunks: Vec<Hunk> = Vec::new();
    let mut cur = Hunk::default();
    let mut flush = |cur: &mut Hunk| -> Result<(), Error> {
        let hunk = std::mem::take(cur);
        if hunk.is_blank() {
            return Ok(());
        }
        if hunk.old == hunk.new {
            return Err(Error::NoChange {
                path: path.to_string(),
                hunk: hunks.len() + 1,
            });
        }
        hunks.push(hunk);
        Ok(())
    };
    while i < until && !is_section(lines[i]) && lines[i] != END {
        let line = lines[i];
        if line == EOF_MARK {
            cur.eof = true;
            flush(&mut cur)?;
        } else if line == "@@" {
            flush(&mut cur)?;
        } else if let Some(anchor) = line.strip_prefix("@@ ") {
            // An anchor after body lines opens the next hunk.
            if !cur.old.is_empty() || !cur.new.is_empty() {
                flush(&mut cur)?;
            }
            cur.anchors.push(anchor.to_string());
        } else if let Some(rest) = line.strip_prefix('+') {
            cur.new.push(rest.to_string());
        } else if let Some(rest) = line.strip_prefix('-') {
            cur.old.push(rest.to_string());
        } else if let Some(rest) = line.strip_prefix(' ') {
            cur.old.push(rest.to_string());
            cur.new.push(rest.to_string());
        } else if line.is_empty() {
            cur.old.push(String::new());
            cur.new.push(String::new());
        } else {
            return Err(Error::BadLine {
                line: i + 1,
                content: line.to_string(),
            });
        }
        i += 1;
    }
    flush(&mut cur)?;
    if hunks.is_empty() {
        return Err(Error::EmptyUpdate {
            path: path.to_string(),
        });
    }
    let op = FileOp::Update {
        path: path.to_string(),
        move_to,
        hunks,
    };
    Ok((op, i))
}

/// One envelope, one author per path (§2.5 discipline in miniature): a
/// path named by two operations — as a source or as a rename target —
/// would make the result order-dependent, so it is declined.
fn check_duplicates(ops: &[FileOp]) -> Result<(), Error> {
    let mut seen = std::collections::BTreeSet::new();
    for op in ops {
        let paths: Vec<&String> = match op {
            FileOp::Add { path, .. } | FileOp::Delete { path } => vec![path],
            FileOp::Update { path, move_to, .. } => {
                std::iter::once(path).chain(move_to.iter()).collect()
            }
        };
        for path in paths {
            if !seen.insert(path.clone()) {
                return Err(Error::DuplicatePath { path: path.clone() });
            }
        }
    }
    Ok(())
}
