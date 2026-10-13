//! **What each provider row offers, as the tuning pane asked it** (bl-b7d9).
//!
//! A role left on a model id its provider does not offer used to be painted
//! like every healthy row, and the editor that mends it was two bare text
//! boxes — while `lernie model` (`crate::seat::model`) already asked the row
//! what it offers before writing. This is the pane spending that same read: the
//! `models` op and its `Reply::Models` decode, one authority for both.
//!
//! # Absence is not evidence
//!
//! The CLI's rule verbatim: a listing that refuses, does not arrive, or is
//! empty says nothing, so a row whose provider has no entry here is marked
//! nothing. Only a row whose provider answered a listing that does not hold its
//! model is marked — that is the one fact in hand.

use super::{Model, Tuning};
use crate::reply::roles::RoleRow;
use crate::reply::{Read, Reply};

impl Model {
    /// **The provider rows the pane asks what they offer**: every role's, and
    /// the draft's as typed — so naming another row in the editor fetches that
    /// row's list on the next beat. Sorted and once each, because two roles on
    /// one row are one question.
    pub fn tuned(&self) -> Vec<String> {
        let mut rows: Vec<String> = self
            .roles
            .iter()
            .flatten()
            .map(|row| row.provider.clone())
            .collect();
        if let Some(Tuning::Editing(edit)) = &self.tuning {
            rows.push(edit.provider.trim().to_owned());
        }
        rows.retain(|row| !row.is_empty());
        rows.sort();
        rows.dedup();
        rows
    }

    /// **File one row's answer.** A non-empty listing replaces the row's
    /// entry; anything else removes it, because a row that stopped answering
    /// has stopped being evidence.
    pub fn offering(&mut self, provider: &str, read: Read) {
        match read {
            Read::Answer(Reply::Models(rows)) if !rows.is_empty() => {
                self.offers.insert(provider.to_owned(), rows);
            }
            _ => {
                self.offers.remove(provider);
            }
        }
    }

    /// What `provider` offers, or nothing where it has said nothing.
    pub fn offered_by(&self, provider: &str) -> Vec<String> {
        self.offers
            .get(provider.trim())
            .cloned()
            .unwrap_or_default()
    }

    /// **The sentence a row earns when its provider's listing lacks its
    /// model**, or `None` — including where there is no listing at all.
    pub fn unoffered(&self, row: &RoleRow) -> Option<String> {
        let offered = self.offers.get(&row.provider)?;
        (!offered.contains(&row.model))
            .then(|| format!("{} does not offer {}", row.provider, row.model))
    }
}

#[cfg(test)]
mod tests;
