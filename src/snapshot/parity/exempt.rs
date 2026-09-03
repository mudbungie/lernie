//! **The exemption ledger** — `parity.toml`, and the strict subset of TOML it
//! is allowed to be.
//!
//! A deliberate absence is config rather than code (PARITY §7), so that
//! deleting a line re-reddens the gate and no source changes either way. The
//! file is a flat list of `op = "reason"`, which is valid TOML and is also the
//! whole of what this reads: **no crate parses it**, because a dependency for
//! forty lines of `key = "value"` is a dependency taken to avoid writing a
//! `split_once`, and the approved set (`Cargo.toml`) does not carry one.
//!
//! Reading a subset is stricter than reading TOML and that is the point: a
//! table header, an array, a bare value or a stray word is REFUSED and names
//! its line, rather than being parsed into something the ledger does not mean.
//!
//! **Every reason must cite a ball.** A reason is not prose for a reader, it is
//! the answer to *is this still true and who is answering it* — and a citation
//! is the only form of that answer a machine can check is present at all.

use std::path::{Path, PathBuf};

/// One recorded absence.
#[derive(Debug)]
pub(crate) struct Exemption {
    /// The op that has no control here.
    pub(crate) op: String,
    /// Why, citing a ball.
    pub(crate) why: String,
}

/// The citation's shape: a balls id, `bl-` and four of its characters.
const CITE: &str = "bl-";

/// Where the ledger lives — off the manifest, like the corpus, because a
/// test's working directory is not a promise.
pub(crate) fn path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("parity.toml")
}

/// Read the committed ledger, refusing anything the subset does not cover.
pub(crate) fn read() -> Vec<Exemption> {
    let text = std::fs::read_to_string(path()).expect("parity.toml");
    parse(&text).unwrap_or_else(|why| panic!("parity.toml: {why}"))
}

/// **The subset, as a function of the text**, so both of its answers can be
/// asked for directly by the suite.
pub(crate) fn parse(text: &str) -> Result<Vec<Exemption>, String> {
    let mut rows: Vec<Exemption> = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let at = n + 1;
        let (op, rest) = line
            .split_once('=')
            .ok_or_else(|| format!("line {at} is not `op = \"reason\"`"))?;
        let op = op.trim();
        if op.is_empty() || op.split_whitespace().count() != 1 {
            return Err(format!("line {at} names no single op"));
        }
        let quoted = rest.trim();
        let why = quoted
            .strip_prefix('"')
            .and_then(|rest| rest.strip_suffix('"'))
            .ok_or_else(|| format!("line {at}: the reason for {op:?} is not a quoted string"))?;
        if !why.contains(CITE) {
            return Err(format!(
                "line {at}: the reason for {op:?} cites no ball — a reason with no citation \
                 cannot be checked for still being true"
            ));
        }
        if rows.iter().any(|row| row.op == op) {
            return Err(format!("line {at}: {op:?} is recorded twice"));
        }
        rows.push(Exemption {
            op: op.to_owned(),
            why: why.to_owned(),
        });
    }
    // **An empty ledger is not refused**, and that is deliberate rather than a
    // missing direction: a seat that surfaces every control-classed op records
    // nothing here, and a gate that forbade its own success state would be a
    // gate nobody could ever satisfy. A ledger emptied by accident is caught by
    // the coverage assertion reddening for every op it used to hold, which is
    // the loudest failure this file has.
    Ok(rows)
}
