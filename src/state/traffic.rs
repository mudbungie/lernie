//! **What crosses the lock** — a worker's report, and the question set the
//! frame publishes for them.
//!
//! Split from [`super`] at the design-time budget on the seam the module's own
//! doc already draws: [`super`] is the lock and the two sides' one call each,
//! and this is what they say to each other. The first changes when the
//! threading does; the second when a pane learns to ask something new.

use serde_json::Value;

use crate::channel::Reach;
use crate::reply::Read;
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

/// What a leg produced. **More than one failure outcome**, because a channel
/// this box cannot open is a different sentence from an engine that refused:
/// the first is about this box's own files or the far end being down, the
/// second is the engine answering. A seat that read them alike would send an
/// operator to check a certificate over a workspace name they mistyped.
///
/// The fourth arm is that division taken one step further (bl-3969): what a
/// failed leg means depends on whether the gesture was an ACT, so the poster
/// reports one and the two read workers report the other.
#[derive(Debug, Clone)]
pub enum Said {
    /// One reply frame, exactly as it crossed.
    Frame(Value),
    /// **The held read's accumulation so far, stamped with what it is about.**
    ///
    /// The stamp is what makes a stale tail impossible rather than unlikely.
    /// The engine was asked about a conversation and answers about that one, so
    /// only this end knows the focus has moved — and only the FRAME knows what
    /// it is looking at right now. So the lane says what its frames are about
    /// and the frame decides whether they are still wanted, which is a pure
    /// comparison at the one place that holds the answer, rather than a poll
    /// racing the socket at the other.
    ///
    /// **Already read**, unlike [`Frame`](Self::Frame): a follow frame is an
    /// append (REMOTE §5.5), so it has to be absorbed onto the read's own fold
    /// before it means anything, and the lane is where a read begins and ends.
    /// What crosses is therefore the whole tail, which is what lets the model
    /// go on replacing.
    Live { conversation: String, read: Read },
    /// **A sign-in lane's fold so far, stamped with the row it is about**
    /// (REMOTE §8.3, bl-e3c5) — [`Live`](Self::Live)'s shape one noun over,
    /// and stamped for the identical reason: the engine was asked about one
    /// provider and answers about that one, so only the FRAME knows whether the
    /// operator is still following it. A run started on a second row replaces
    /// the first upstream, and an unstamped frame would paint one run's lines
    /// under the other's name.
    Signin { provider: String, read: Read },
    /// This seat could not reach the far end, and here is the sentence.
    ///
    /// **A READ's failure, and only a read's.** It is a fact about a
    /// relationship — this channel is not answering — which is why it lands on
    /// that channel's own roster section rather than in the shell's bar
    /// (REMOTE §8.2, bl-e620).
    Unreachable(String),
    /// **An ACT that earned no reply**, and what the transport can say about
    /// whether it crossed (REMOTE §3, bl-3969).
    ///
    /// A fact about an **exchange** and not about a relationship, so it goes to
    /// the bar where a refusal goes. The `op` rides with it because the bar is
    /// one line for the whole window: a sentence about an act that does not
    /// name the act is a sentence about nothing an operator can act on.
    Acted { op: String, reach: Reach },
    /// **A routed gesture's reply frame**, stamped with the op it answers
    /// (bl-b180).
    ///
    /// A refusal wears no `kind`, so a frame alone cannot say which exchange
    /// it closes — and the one thing this window holds across two acts, a
    /// start, has to know whether the sentence that came back is *its* refusal
    /// or somebody else's. The poster is the one place that still knows, for
    /// [`Acted`](Self::Acted)'s reason exactly, so it says so on the way in
    /// rather than the model guessing from timing. A standing read's frame
    /// stays a bare [`Frame`](Self::Frame): it is answered in place and
    /// nothing is held against it.
    Receipt { op: String, frame: Value },
}

/// **Which covering pane's standing read is up.**
///
/// It is keyed on the PANE rather than on a focus, for one reason all four
/// share: the reads are cheap, and a standing question nobody has a use for is
/// still a question the engine answers on every beat, forever. The queue's is
/// the sharpest — a read that fans costs one round trip per channel per beat —
/// and the login pane's is the one whose standing actually buys something, a
/// credential landing on the engine while the operator is looking at the table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Open {
    /// **The tuning pane** — the aimed wall's fourth question, what its roles
    /// are set to (bl-4a2c).
    Tuning,
    /// **The records pane** — the selected conversation's own second question,
    /// what its loop did and what its worktree holds (bl-2cf7).
    Records,
    /// **The decision queue** — the one question that is nobody's focus:
    /// `attention` names no workspace, so it is asked of every channel rather
    /// than of the aim (bl-f0ef).
    Queue,
    /// **The trail** — the second question that is nobody's focus: `ops`
    /// names no workspace either, so it is asked of every channel rather than
    /// of the aim, and it stands because a trail is what is happening
    /// (bl-4c48).
    Trail,
    /// **The clients pane** — the aimed wall's sixth question, the machines
    /// registered in it and what each offers (bl-e53c). Its standing buys the
    /// same thing the login pane's does: presence is true only at the moment
    /// it was answered (REMOTE §5), so a row that said *not connected* says
    /// otherwise on the next beat with nothing asked again.
    Clients,
    /// **The config pane** — the aimed wall's seventh question: the lineages
    /// it holds, and the file the pane is pointed at (bl-5c53). The
    /// destination rides *inside* the arm for [`Open::Login`]'s reason: there
    /// is no file to read while the pane is down, so a field beside it would
    /// be a second authority for one fact.
    Config(Option<crate::verbs::Where>),
    /// **The login pane** — the aimed wall's fifth question, what it can sign
    /// in to, carrying the provider row whose sign-in the held lane is on where
    /// one has been started (bl-e3c5). The row rides *inside* the pane's own
    /// arm because there is no sign-in to follow while the pane is down, so a
    /// field beside it would be a second authority for one fact.
    Login(Option<String>),
}

impl Open {
    /// Which of them the model has standing. **A ladder rather than a set**,
    /// because the model's own `covered` question already promises no two are
    /// open at once — the order is a total function over a state space the
    /// window cannot reach rather than a precedence anybody spends.
    fn of(model: &Model) -> Option<Self> {
        if model.tuning.is_some() {
            Some(Self::Tuning)
        } else if model.showing(crate::ui::Listing::Records) {
            Some(Self::Records)
        } else if model.showing(crate::ui::Listing::Queue) {
            Some(Self::Queue)
        } else if model.trailing() {
            Some(Self::Trail)
        } else if model.login.is_some() {
            Some(Self::Login(model.following()))
        } else if model.showing(crate::ui::Listing::Clients) {
            Some(Self::Clients)
        } else if model.configuring.is_some() {
            Some(Self::Config(model.configured()))
        } else {
            None
        }
    }
}

/// **What to ask next.** Derived from the model on every settle, so it cannot
/// drift from the focus and there is nothing to invalidate.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Standing {
    /// Every channel this seat holds: each is asked for its own roster.
    pub channels: Vec<Channel>,
    /// The aimed wall, whose conversations are the second question.
    pub aim: Option<Aim>,
    /// **Which covering pane's standing read is up**, if any.
    ///
    /// One field rather than a flag per pane, which is `crate::ui::Lookup`'s
    /// reframe one layer down and the one clippy's `struct_excessive_bools`
    /// asks for by name. No two covering panes ever stand together
    /// (`crate::ui::Model::covered`), so four bools would make *three of them
    /// open at once* a representable state that only the derivation order
    /// resolves — two representations of one fact.
    pub open: Option<Open>,
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
            open: Open::of(model),
            conversation: model.asked(),
        }
    }

    /// **Whether `pane`'s standing read is the one that is up.**
    ///
    /// The asker's whole reading of [`Self::open`], so a pass tests a pane
    /// rather than matching an enum four times.
    pub fn standing(&self, pane: &Open) -> bool {
        self.open.as_ref() == Some(pane)
    }

    /// **The file the config pane is pointed at**, if it is open and a
    /// destination has been picked on it — [`Self::signin`]'s shape one pane
    /// over, and for its reason: the read that stands is the pane's own
    /// question, asked here so the pass tests one thing.
    pub fn at(&self) -> Option<crate::verbs::Where> {
        match &self.open {
            Some(Open::Config(at)) => at.clone(),
            _ => None,
        }
    }

    /// **The provider row the sign-in lane is on**, if the login pane is open
    /// and a sign-in has been started from it.
    pub fn signin(&self) -> Option<String> {
        match &self.open {
            Some(Open::Login(provider)) => provider.clone(),
            _ => None,
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
