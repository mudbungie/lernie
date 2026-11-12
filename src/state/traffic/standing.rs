//! **The question set the frame publishes**, and which panes' standing
//! reads are in it. Split from [`super`] at the 300-line cap on the seam
//! that file's own doc draws: there is what a worker says, and here is what
//! the frame asks.

use crate::ui::{Aim, Channel, Model};

/// **Which covering pane's standing read is up.**
///
/// It is keyed on the PANE rather than on a focus, for one reason all four
/// share: the reads are cheap, and a standing question nobody has a use for is
/// still a question the server answers on every beat, forever. The queue's is
/// the sharpest — a read that fans costs one round trip per channel per beat —
/// and the login pane's is the one whose standing actually buys something, a
/// credential landing on the server while the operator is looking at the table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Open {
    /// **The tuning pane** — the aimed workspace's fourth question, what its roles
    /// are set to (bl-4a2c), carrying the provider rows its rows and draft
    /// name, each asked what it offers on the same standing (bl-b7d9).
    Tuning(Vec<String>),
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
    /// **The ball pane** — the third question that is nobody's focus, and
    /// the one that is BOTH: `balls` and `board` name no workspace, so they
    /// are asked of every channel, and `workspace-balls` and `marks` name the
    /// aimed workspace, so they are asked of it while the same pane stands
    /// (bl-d2af). It stands for the trail's reason exactly — a board is what
    /// is happening.
    Board,
    /// **The fleet pane** — the aimed workspace's seventh question, its delivery
    /// attempts and what its agents changed (bl-a43a). Both are DERIVED when
    /// asked — nothing behind either is stored — so the same row a minute
    /// later is a statement about the world a minute later, which is the
    /// trail's reason for standing read on a subject that is a focus.
    Fleet,
    /// **The clients pane** — the aimed workspace's sixth question, the machines
    /// registered in it and what each offers (bl-e53c). Its standing buys the
    /// same thing the login pane's does: presence is true only at the moment
    /// it was answered (REMOTE §5), so a row that said *not connected* says
    /// otherwise on the next beat with nothing asked again.
    Clients,
    /// **The offers page** — the aimed workspace's pending offers, standing
    /// for the clients pane's reason: a row moves from open to claimed to
    /// proved on the server with nothing asked here (DESIGN §4.51).
    Offers,
    /// **The config pane** — the aimed workspace's seventh question: the lineages
    /// it holds, the file the pane is pointed at (bl-5c53), and the staged
    /// proposals at the depth of the one it has named (bl-a1d6). Both ride
    /// *inside* the arm for [`Open::Login`]'s reason: there is no file and no
    /// proposal to read while the pane is down, so a field beside it would be
    /// a second authority for one fact.
    Config(Option<crate::verbs::Where>, Option<String>),
    /// **The login pane** — the aimed workspace's fifth question, what it can sign
    /// in to, carrying the provider row whose sign-in the held lane is on where
    /// one has been started (bl-e3c5). The row rides *inside* the pane's own
    /// arm because there is no sign-in to follow while the pane is down, so a
    /// field beside it would be a second authority for one fact.
    Login(Option<String>),
}

impl Open {
    /// Which of them the model has standing. **A set, keyed on being
    /// shown wherever shown** (DESIGN §4.48 item 8): no two floating panes
    /// stand together, but a docked one stands beside whatever floats, and
    /// its read stands with it.
    fn of(model: &Model) -> Vec<Self> {
        let mut open = Vec::new();
        if model.tuning.is_some() {
            open.push(Self::Tuning(model.tuned()));
        }
        if model.recording() {
            open.push(Self::Records);
        }
        if model.showing(crate::ui::Listing::Queue) {
            open.push(Self::Queue);
        }
        if model.trailing() {
            open.push(Self::Trail);
        }
        if model.boarding() {
            open.push(Self::Board);
        }
        if model.fleet.is_some() {
            open.push(Self::Fleet);
        }
        if model.login.is_some() {
            open.push(Self::Login(model.following()));
        }
        if model.showing(crate::ui::Listing::Clients) {
            open.push(Self::Clients);
        }
        if model.showing(crate::ui::Listing::Offers) {
            open.push(Self::Offers);
        }
        if model.configuring().is_some() {
            open.push(Self::Config(model.configured(), model.proposing()));
        }
        open
    }
}

/// **What to ask next.** Derived from the model on every settle, so it cannot
/// drift from the focus and there is nothing to invalidate.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Standing {
    /// Every channel this seat holds: each is asked for its own roster.
    pub channels: Vec<Channel>,
    /// The aimed workspace, whose conversations are the second question.
    pub aim: Option<Aim>,
    /// **Which panes' standing reads are up** — one while a pane floats
    /// alone, more while the rails stand beside one (§4.48 item 8).
    pub open: Vec<Open>,
    /// The selected conversation, whose transcript is the third — and whose
    /// live tail is the held read.
    ///
    /// **Not every selection is one.** A conversation this window has just
    /// started is selected under a name the server resolves nowhere until its
    /// driver writes the branch, and asking about it would earn a refusal per
    /// pass for the whole of a healthy start. `Model::asked` is the reading
    /// that leaves it out (`crate::ui::model::claim`).
    pub conversation: Option<String>,
    /// **Whether the selected conversation is asking** (`crate::ui::ask`):
    /// the queue's read stands while it does, so the ask above the prompt
    /// can be answered on the frame the queue's row lands. The roster's own
    /// word, read before the queue has answered.
    pub asking: bool,
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
            asking: model.asks(),
        }
    }

    /// **Whether `pane`'s standing read is the one that is up.**
    ///
    /// The asker's whole reading of [`Self::open`], so a pass tests a pane
    /// rather than matching an enum four times.
    pub fn standing(&self, pane: &Open) -> bool {
        self.open.contains(pane)
    }

    /// **The file the config pane is pointed at**, if it is open and a
    /// destination has been picked on it — [`Self::signin`]'s shape one pane
    /// over, and for its reason: the read that stands is the pane's own
    /// question, asked here so the pass tests one thing.
    pub fn at(&self) -> Option<crate::verbs::Where> {
        self.open.iter().find_map(|open| match open {
            Open::Config(at, _) => at.clone(),
            _ => None,
        })
    }

    /// **The proposal the config pane has named**, if it is open and one is
    /// named and still staged — the depth the proposals read is asked at.
    pub fn named(&self) -> Option<String> {
        self.open.iter().find_map(|open| match open {
            Open::Config(_, id) => id.clone(),
            _ => None,
        })
    }

    /// **The provider row the sign-in lane is on**, if the login pane is open
    /// and a sign-in has been started from it.
    pub fn signin(&self) -> Option<String> {
        self.open.iter().find_map(|open| match open {
            Open::Login(provider) => provider.clone(),
            _ => None,
        })
    }

    /// **The provider rows the tuning pane asks what they offer**, while it
    /// stands.
    pub fn tuned(&self) -> Option<&Vec<String>> {
        self.open.iter().find_map(|open| match open {
            Open::Tuning(providers) => Some(providers),
            _ => None,
        })
    }

    /// The channel the aim is on, when there is one and this seat still holds
    /// it. A focus on a channel that has since gone is not a question.
    pub fn aimed(&self) -> Option<(Channel, Aim)> {
        let aim = self.aim.clone()?;
        let held = self.channels.iter().find(|held| held.name == aim.channel)?;
        Some((held.clone(), aim))
    }
}
