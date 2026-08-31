//! **What crosses the lock** — a worker's report, and the question set the
//! frame publishes for them.
//!
//! Split from [`super`] at the design-time budget on the seam the module's own
//! doc already draws: [`super`] is the lock and the two sides' one call each,
//! and this is what they say to each other. The first changes when the
//! threading does; the second when a pane learns to ask something new.

use serde_json::Value;

use crate::ui::{Aim, Channel, Model};

/// **What one worker heard**, stamped with the channel it came down.
///
/// The stamp is the client's and it is applied here, where the worker that
/// opened the channel still knows which one it was: no origin crosses the wire,
/// and a frame that arrived with no way back to its channel could not be filed
/// against the right roster section.
#[derive(Debug, Clone)]
pub struct Heard {
    pub channel: Channel,
    pub said: Said,
}

/// What a leg produced. **Three outcomes and not two**, because a channel this
/// box cannot open is a different sentence from an engine that refused: the
/// first is about this box's own files or the far end being down, the second is
/// the engine answering. A seat that read them alike would send an operator to
/// check a certificate over a workspace name they mistyped.
#[derive(Debug, Clone)]
pub enum Said {
    /// One reply frame, exactly as it crossed.
    Frame(Value),
    /// **One frame of the held read, stamped with what it is about.**
    ///
    /// The stamp is what makes a stale tail impossible rather than unlikely.
    /// The engine was asked about a conversation and answers about that one, so
    /// only this end knows the focus has moved — and only the FRAME knows what
    /// it is looking at right now. So the lane says what its frames are about
    /// and the frame decides whether they are still wanted, which is a pure
    /// comparison at the one place that holds the answer, rather than a poll
    /// racing the socket at the other.
    Live { conversation: String, frame: Value },
    /// This seat could not reach the far end, and here is the sentence.
    Unreachable(String),
}

/// **What to ask next.** Derived from the model on every settle, so it cannot
/// drift from the focus and there is nothing to invalidate.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Standing {
    /// Every channel this seat holds: each is asked for its own roster.
    pub channels: Vec<Channel>,
    /// The aimed wall, whose conversations are the second question.
    pub aim: Option<Aim>,
    /// The selected conversation, whose transcript is the third — and whose
    /// live tail is the held read.
    ///
    /// **Not every selection is one.** A conversation this window has just
    /// started is selected under a name the engine resolves nowhere until its
    /// driver writes the branch, and asking about it would earn a refusal per
    /// pass for the whole of a healthy start. `Model::asked` is the reading
    /// that leaves it out (`crate::ui::model::claim`).
    pub conversation: Option<String>,
}

impl Standing {
    /// The question set this model implies.
    pub fn of(model: &Model) -> Self {
        Self {
            channels: model
                .roster
                .iter()
                .map(|chunk| chunk.channel.clone())
                .collect(),
            aim: model.aim.clone(),
            conversation: model.asked(),
        }
    }

    /// The channel the aim is on, when there is one and this seat still holds
    /// it. A focus on a channel that has since gone is not a question.
    pub fn aimed(&self) -> Option<(Channel, Aim)> {
        let aim = self.aim.clone()?;
        let held = self.channels.iter().find(|held| held.name == aim.channel)?;
        Some((held.clone(), aim))
    }
}
