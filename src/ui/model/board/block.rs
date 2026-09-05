//! **The block a ball's text is authored in** (bl-f7ae; DESIGN §4.35) — what
//! it holds between frames, and the four gestures it composes.
//!
//! Split from [`super::acts`] at the design-time budget on the seam the config
//! pane's own draft already draws (`super::super::config::draft`): that module
//! is what the MODEL does — open it, close it, send what it composed — and
//! this is the one value those doors are about.
//!
//! # One block, two subjects
//!
//! A ball that does not exist yet and a ball this wall holds. They are one
//! block because authoring a ball's words is one act, which is the fold
//! upstream's own `actions/verbs/balls/edit` already made — *"one vocabulary,
//! so a fact balls learns is added in one place instead of in the roster, the
//! codec's field list and a second struct beside them"*.
//!
//! # The enablement and the gesture are one fact
//!
//! Each composer answers `Option<Value>`, and its `None` is exactly what makes
//! the control that spends it dark. A seat with a bool beside each envelope
//! would hold two representations of *can this go*, and the one that drifted
//! would be the one nobody clicked.

use serde_json::Value;

use super::super::Aim;
use crate::reply::balls::BoundBall;

/// **A ball's text being authored** — one that does not exist yet, or one the
/// aimed wall holds.
///
/// One block with two subjects rather than two blocks, because it is one
/// subject: authoring a ball's words. Upstream folds them the same way and
/// says why — *"one vocabulary, so a fact balls learns is added in one place
/// instead of in the roster, the codec's field list and a second struct beside
/// them"*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Authoring {
    /// The wall it was opened on, held rather than followed — see the module
    /// doc.
    pub at: Aim,
    /// **The name its acts stamp `--as`**, taken once at the moment the block
    /// opened, for the same reason the wall is.
    pub name: String,
    /// The ball this is about, or `None` for one that does not exist yet.
    /// Every field a gesture about an existing ball needs is on it.
    pub ball: Option<BoundBall>,
    /// **Where a NEW ball is filed.** Typed, because nothing on this pane
    /// names a project for a ball that does not exist — a wall's project is
    /// not a fact on this wire. An existing ball carries its own.
    pub project: String,
    /// What it is called.
    pub title: String,
    /// The rest of the description.
    pub body: String,
    /// **What to append to its journal**, which is why it is not the body: yog
    /// appends a note and replaces a body.
    pub note: String,
    /// **The id typed back**, which is the arming `close` takes (§4.20). It
    /// names its own subject, so nothing else records which ball is armed.
    pub arm: String,
}

impl Authoring {
    /// A block opened on a wall, about a ball or about none.
    pub(super) fn of(at: Aim, name: String, ball: Option<BoundBall>) -> Self {
        Self {
            project: ball
                .as_ref()
                .map(|held| held.project.clone())
                .unwrap_or_default(),
            at,
            name,
            ball,
            title: String::new(),
            body: String::new(),
            note: String::new(),
            arm: String::new(),
        }
    }

    /// **Whether it is about a ball that does not exist yet** — the one
    /// question the paint asks to know which boxes belong on the glass.
    pub fn filing(&self) -> bool {
        self.ball.is_none()
    }

    /// **File it.** `None` where the block is about an existing ball, or where
    /// a project or a title is missing — which is the enablement and the
    /// gesture read off one fact rather than two.
    pub fn filed(&self) -> Option<Value> {
        if self.ball.is_some() {
            return None;
        }
        let (project, title) = (said(&self.project)?, said(&self.title)?);
        Some(crate::verbs::create(
            project,
            self.name.clone(),
            title,
            said(&self.body),
        ))
    }

    /// **Amend it.** `None` for a new ball, and `None` where nothing was
    /// typed: upstream refuses an update that changes nothing by name, so a
    /// control that could send one would be spending a round trip on a
    /// refusal this end can see coming.
    pub fn amended(&self) -> Option<Value> {
        let ball = self.ball.as_ref()?;
        let (title, body, note) = (said(&self.title), said(&self.body), said(&self.note));
        if title.is_none() && body.is_none() && note.is_none() {
            return None;
        }
        Some(crate::verbs::update(
            ball.project.clone(),
            ball.id.clone(),
            self.name.clone(),
            title,
            body,
            note,
        ))
    }

    /// **Let it go.** The undoing of a claim, so nothing arms it.
    pub fn released(&self) -> Option<Value> {
        let ball = self.ball.as_ref()?;
        Some(crate::verbs::release(
            ball.project.clone(),
            ball.id.clone(),
            self.name.clone(),
        ))
    }

    /// **Deliver it**, and `None` until the arming box holds this ball's own
    /// id (§4.20: *the arming is the subject's own name*).
    pub fn delivered(&self) -> Option<Value> {
        let ball = self
            .ball
            .as_ref()
            .filter(|held| said(&self.arm) == Some(held.id.clone()))?;
        Some(crate::verbs::close(
            ball.project.clone(),
            ball.id.clone(),
            self.name.clone(),
        ))
    }
}

/// **What was typed, or nothing at all.** Trimmed, because leading and
/// trailing space is typing; empty is absent, which is a value on both
/// authoring doors (`crate::verbs::balls::edit`).
fn said(typed: &str) -> Option<String> {
    Some(typed.trim())
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
}

#[cfg(test)]
mod tests;
