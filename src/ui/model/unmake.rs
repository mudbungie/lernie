//! **An unmaking between frames** (bl-48fa): what is being unmade, what has
//! been typed to arm it, and whether it has been asked for.
//!
//! Its type and its three acts share one file for the reason
//! [`super::tuning`]'s do — `super::acts` is at its design-time budget, and
//! these acts have no reader but this type. The rule that governs the placement
//! is the same either way: a binding names a control that already exists, so
//! what a click means lives once.
//!
//! # It holds its own subject, and that is the safety property
//!
//! [`Unmaking`] carries the [`Aim`] it was opened on rather than reading
//! [`Model::aim`] when it fires, which is the opposite of what the tuning pane
//! does and is the right answer for the opposite reason. The roster stays live
//! and clickable under a covering pane, so an aim can move while an unmaking
//! stands — and a destructive act that followed the aim would unmake a wall the
//! operator armed a different one for. A tuning write that lands on the wrong
//! wall is undone by writing the old value back; this is not.
//!
//! # The arming is an enablement, and it is never spent
//!
//! [`Unmaking::armed`] is the readiness test, and it is the wire's own rule
//! rather than a policy invented here: `delete-workspace` is refused unless the
//! typed name matches the workspace exactly (`crate::verbs::DELETE_WORKSPACE`).
//! Firing does not clear it, because a refusal is the COMMON case for this act —
//! the engine declines while anything in the workspace is live — and clearing
//! the box would charge a retype for the engine's *no*, which is a toll on the
//! safe path. `crate::ui::composer::acts` draws the same distinction one noun
//! over, where the same field is a parameter instead.

use serde_json::Value;

use super::{Aim, Model};

/// **A workspace being unmade**, from the control that opened the pane to the
/// control that closes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unmaking {
    /// The wall this is about, held rather than re-read (see the module doc).
    pub aim: Aim,
    /// The arming: the workspace's own name, typed back.
    pub typed: String,
    /// **Whether it has been asked for.** The gesture a frame composes is
    /// drained within the beat, so the outbox cannot answer *has this been
    /// asked* — and an operator with no answer to that clicks again.
    pub posted: bool,
}

impl Unmaking {
    /// Open one on a wall, unarmed.
    pub fn at(aim: Aim) -> Self {
        Self {
            aim,
            typed: String::new(),
            posted: false,
        }
    }

    /// **Whether the typed name arms it** — the workspace's own name and
    /// nothing else, which is what the engine compares against.
    pub fn armed(&self) -> bool {
        self.typed == self.aim.address
    }

    /// The gesture it composes, which is the same object `lernie
    /// delete-workspace` builds.
    pub fn gesture(&self) -> Value {
        crate::verbs::delete_workspace(self.aim.address.clone(), self.typed.clone())
    }
}

impl Model {
    /// **Open an unmaking on the wall the window is aimed at**, or do nothing
    /// where it is aimed at none — the aim is the gate exactly as it is for the
    /// enrollment and the tuning pane, because the gesture carries a workspace
    /// and a workspace is what an aim is.
    pub fn begin_unmaking(&mut self) {
        if let Some(aim) = self.aim.clone() {
            self.unmaking = Some(Unmaking::at(aim));
        }
    }

    /// **Put it down.** The way out of the pane, and it unmakes nothing.
    pub fn close_unmaking(&mut self) {
        self.unmaking = None;
    }

    /// **Ask for it**, where it is armed and never otherwise. The pane stands
    /// afterwards: there is nothing else on the glass that would tell an
    /// operator the act is in flight, and a refusal is what usually comes back.
    pub fn post_unmaking(&mut self) {
        let Some(unmaking) = self.unmaking.as_mut().filter(|held| held.armed()) else {
            return;
        };
        unmaking.posted = true;
        let gesture = unmaking.gesture();
        self.outbox.push(gesture);
    }
}

#[cfg(test)]
mod tests;
