//! **The offer's replies** (yog's `docs/REMOTE.md` §15.1; DESIGN §4.51):
//! what `offer` answers, and one row of what `offers` lists. `approve` and
//! `decline` answer a bare receipt each and carry nothing to read.
//!
//! None of it is a secret, which is the point of the offer: the URL is a
//! one-time code and the word is a check, so every field here is painted,
//! printed and logged like any other reply.

use serde_json::{Map, Value};

use super::fields;

/// `offer`'s reply kind.
pub(crate) const OFFERED: &str = "offered";
/// `offers`' reply kind.
pub(crate) const KIND: &str = "offers";
/// `approve`'s and `decline`'s.
pub(crate) const APPROVED: &str = "approved";
pub(crate) const DECLINED: &str = "declined";

/// The state only which an offer can be approved in: its device has dialled.
pub const PROVED: &str = "proved";

/// A minted offer: the URL the new device opens, and when its code expires.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offered {
    pub url: String,
    /// Unix seconds.
    pub expires: i64,
}

/// One pending offer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfferRow {
    pub name: String,
    pub grade: String,
    /// `open`, `claimed` or `proved` — or a word this build has not met,
    /// which rides verbatim and can only make a row unapprovable.
    pub state: String,
    /// Unix seconds: when the code expires. It bounds an open offer only.
    pub expires: i64,
    /// The confirmation word, absent until a device has claimed.
    pub word: Option<String>,
}

impl OfferRow {
    /// Whether `approve` would be accepted: the device has dialled.
    pub fn proved(&self) -> bool {
        self.state == PROVED
    }

    /// The row as one line: who, what grade, where it stands, and the word.
    pub fn line(&self) -> String {
        let word = self
            .word
            .as_ref()
            .map_or_else(String::new, |word| format!("  word {word}"));
        format!("{}  {}  {}{word}", self.name, self.grade, self.state)
    }
}

pub(crate) fn offered(obj: &Map<String, Value>) -> Result<Offered, String> {
    Ok(Offered {
        url: fields::text(obj, "url")?,
        expires: fields::secs(obj, "expires")?,
    })
}

pub(crate) fn row(value: &Value) -> Result<OfferRow, String> {
    let obj = value.as_object().ok_or("offer: not a JSON object")?;
    Ok(OfferRow {
        name: fields::text(obj, "name")?,
        grade: fields::text(obj, "grade")?,
        state: fields::text(obj, "state")?,
        expires: fields::secs(obj, "expires")?,
        word: fields::opt_text(obj, "word")?,
    })
}

/// **A moment as a clock reads it**: `HH:MM UTC`, for an expiry ten minutes
/// out. The date is left off on purpose — a code's whole life is shorter
/// than the minute hand's lap.
pub fn clock(unix: i64) -> String {
    let minutes = unix.div_euclid(60);
    format!(
        "{:02}:{:02} UTC",
        minutes.div_euclid(60).rem_euclid(24),
        minutes.rem_euclid(60)
    )
}

#[cfg(test)]
mod tests;
