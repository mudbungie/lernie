//! The **matching ladder** (ARCH §3.3 *The patch tool*): how a hunk's
//! context is located in the target file.
//!
//! Four rungs, tried strictly in order, mirroring the fuzz behavior of
//! `git apply` as codex's `seek_sequence` implements it: exact match;
//! ignore trailing whitespace; ignore edge (leading + trailing)
//! whitespace; Unicode-normalized (smart quotes, NBSP-class spaces and
//! typographic dashes folded to ASCII, then edge whitespace ignored).
//! A rung is consulted only when every rung above it found *nothing*:
//! zero matches descends, one match wins, and more than one match is an
//! ambiguity decline at that rung — never a guessed edit (bl-e249). The
//! ladder therefore recovers from the classic single-smart-quote miss
//! without ever widening what "unique" means.

use thiserror::Error;

/// One rung of the ladder, in descent order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rung {
    Exact,
    TrailingWs,
    EdgeWs,
    Normalized,
}

/// The ladder itself: descent order is declaration order.
pub const LADDER: [Rung; 4] = [
    Rung::Exact,
    Rung::TrailingWs,
    Rung::EdgeWs,
    Rung::Normalized,
];

impl Rung {
    /// The rung's name as reported to the model (and in declines).
    pub fn label(self) -> &'static str {
        match self {
            Rung::Exact => "exact",
            Rung::TrailingWs => "ignore-trailing-whitespace",
            Rung::EdgeWs => "ignore-edge-whitespace",
            Rung::Normalized => "unicode-normalized",
        }
    }

    /// Canonicalize one line for comparison at this rung.
    fn canon(self, s: &str) -> String {
        match self {
            Rung::Exact => s.to_string(),
            Rung::TrailingWs => s.trim_end().to_string(),
            Rung::EdgeWs => s.trim().to_string(),
            Rung::Normalized => normalize(s).trim().to_string(),
        }
    }
}

/// Fold the Unicode look-alikes models (and editors) substitute for
/// ASCII punctuation: typographic dashes, curly quotes, and the
/// non-breaking-space family. The table is deliberately small — it
/// answers the observed failure class, not Unicode at large.
fn normalize(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '\u{2010}'..='\u{2015}' | '\u{2212}' => '-',
            '\u{2018}'..='\u{201B}' => '\'',
            '\u{201C}'..='\u{201F}' | '\u{00AB}' | '\u{00BB}' => '"',
            '\u{00A0}' | '\u{2000}'..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}' => ' ',
            c => c,
        })
        .collect()
}

/// Why the ladder produced no unique target.
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    /// No rung matched anywhere in the searched region.
    #[error(
        "not found at any rung (tried exact, ignore-trailing-whitespace, ignore-edge-whitespace, unicode-normalized)"
    )]
    NotFound,
    /// The first rung to match at all matched more than once. `rung` is
    /// where the tie happened; nothing below it was consulted.
    #[error("{count} matches at the {rung} rung", rung = .rung.label())]
    Ambiguous { rung: Rung, count: usize },
}

/// Locate `needle` in `hay[start..]` and return its position plus the
/// rung that won. `eof` first tries the block flush against the end of
/// the file (every rung), then falls back to the ordinary scan — the
/// `*** End of File` marker prefers the end but does not require it.
/// An empty needle locates the pure-insertion point: the end when
/// `eof`, else `start` (the caller guarantees an anchor put it there).
pub fn seek(
    hay: &[String],
    needle: &[String],
    start: usize,
    eof: bool,
) -> Result<(usize, Rung), Error> {
    if needle.is_empty() {
        let at = if eof { hay.len() } else { start };
        return Ok((at, Rung::Exact));
    }
    let Some(last) = hay.len().checked_sub(needle.len()) else {
        return Err(Error::NotFound);
    };
    if start > last {
        return Err(Error::NotFound);
    }
    if eof {
        for rung in LADDER {
            if matches_at(hay, needle, last, rung) {
                return Ok((last, rung));
            }
        }
    }
    for rung in LADDER {
        let mut hits = (start..=last).filter(|&pos| matches_at(hay, needle, pos, rung));
        if let Some(first) = hits.next() {
            let count = 1 + hits.count();
            if count > 1 {
                return Err(Error::Ambiguous { rung, count });
            }
            return Ok((first, rung));
        }
    }
    Err(Error::NotFound)
}

/// Whole-block comparison at one position under one rung.
fn matches_at(hay: &[String], needle: &[String], pos: usize, rung: Rung) -> bool {
    needle
        .iter()
        .zip(&hay[pos..pos + needle.len()])
        .all(|(n, h)| rung.canon(n) == rung.canon(h))
}
