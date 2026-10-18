//! **What a channel is**, and what a gesture aimed down one must be addressed
//! as (yog's `docs/REMOTE.md` §8.2).
//!
//! Split from [`super`] at the design-time budget on a seam the two already
//! have: [`super`] is what the window holds *right now* and how a reply changes
//! it, and this is the standing fact about where an answer came from. The
//! second changes when the operator provisions a channel; the first changes
//! every frame.

use crate::reply::roster::WsRow;

/// **One channel this box holds**, as the roster stamps the rows that came down
/// it. The stamp is the client's, applied here: no origin crosses the wire.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Channel {
    /// What this box calls it — an entry's leaf, or the label the flat root
    /// wears. It is the roster's section header.
    pub name: String,
    /// The name that workspace bears **on its host**, or `None` for this box's
    /// own server.
    ///
    /// It is what makes [`Self::address`] exact rather than a guess, and the
    /// fact is §8.2's: an entry resolves by its leaf and by nothing else.
    pub named_there: Option<String>,
    /// **The `host:port` this channel dials**, where this box holds one to
    /// dial — the fact `lernie entries` prints under every row and the window
    /// used to drop (bl-77df).
    ///
    /// Two entries naming one address are two trust relationships that happen
    /// to terminate at one listener (§8.2), which is lawful; an entry naming
    /// the address this box's own server listens on is the same thing by
    /// accident, and it paints every workspace of that server twice. The seat
    /// is the only thing that can see that, and it can see it for free — so
    /// the address goes on the section header and a duplicate is self-evident.
    pub dials: Option<String>,
}

impl Channel {
    /// **The name a gesture must carry to reach `row` down this channel**, or
    /// `None` where this seat holds no name for it.
    ///
    /// Three cases and the third is a real one. This box's own server rewrites
    /// nothing, so a row is addressed by its own name. An entry rewrites its
    /// leaf to the host's name at the channel boundary
    /// ([`crate::seat::route`]), so the leaf is the address of the one
    /// workspace that entry names. And an entry's server may answer a row the
    /// entry does **not** name — a workspace this client is registered in and
    /// holds no entry for — which is reachable by no envelope this seat can
    /// write. It is painted, and painted as unreachable: dropping it would hide
    /// a workspace the operator has, and addressing it by the leaf would aim a
    /// gesture at a different workspace.
    pub fn address(&self, row: &WsRow) -> Option<String> {
        match &self.named_there {
            None => Some(row.workspace.clone()),
            Some(there) if *there == row.workspace => Some(self.name.clone()),
            Some(_) => None,
        }
    }
}

/// **What a section has instead of workspaces, when it has none** (bl-08b6).
///
/// Four states and they are four different facts, which is why an empty
/// section cannot be one blank: a channel nobody has heard from yet, a channel
/// this box cannot dial and knows why off its own files, a server that
/// answered its roster with a refusal (bl-693d), and a server that answered
/// and holds no workspace. The roster used to have a sentence for the
/// empty ROSTER and none for an empty section — and an empty roster is
/// unreachable, because every box holds its own server's slot whether or not
/// anything is provisioned in it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Held {
    /// **Nothing has come down this channel yet.** The [`crate::ui::convs`]
    /// pane's own doctrine, one noun over: an empty list is not evidence that a
    /// thing holds nothing until somebody has looked.
    #[default]
    Unheard,
    /// **This box cannot dial it, and here is why** — the sentence
    /// `crate::channel` already computes and `lernie entries` already prints:
    /// unprovisioned, half-provisioned and naming the missing file, hollow, or
    /// a port only the server that bound it knows.
    Unheld(String),
    /// **It answered, and refused the roster** (bl-693d) — in its own words,
    /// which today are *this client is registered in no workspace here*. The
    /// server is up and dialled, so this is neither of the two above: it is
    /// not *waiting to hear* and not *down*, and the remedy it names — found
    /// a workspace, or be enrolled in one — is a gesture on this section. The
    /// next roster answer spends it, as it spends every other state.
    Refused(String),
    /// It answered.
    Heard,
}

/// **What leads the wire's own sentence for a channel that will not dial**
/// (bl-8a4b): that sentence is `lernie entries`' too, and names files and
/// transports — the lead says what it means for the window first.
pub const UNHELD: &str = "cannot reach this server: ";

impl Held {
    /// **The sentence a section stands on instead of workspaces**, where it has
    /// one: why it will not dial, or what the server refused — the latter in
    /// [`super::Notice`]'s own words for a refusal, so it reads the same on
    /// the section as it did in the bar.
    pub fn said(&self) -> Option<String> {
        match self {
            Self::Unheld(why) => Some(format!("{UNHELD}{why}")),
            Self::Refused(said) => Some(super::Notice::Refused(said.clone()).line()),
            Self::Unheard | Self::Heard => None,
        }
    }
}

/// One channel's answer, as the roster shows it: the section, what it has
/// instead of workspaces when it has none, its currency, and its workspaces.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Chunk {
    pub channel: Channel,
    /// What this section has instead of workspaces — read at boot off this box's own
    /// files, and spent by the first answer that comes down the channel.
    pub held: Held,
    /// How stale the derivation behind these rows is, when the server said.
    pub stale: Option<String>,
    /// What grew since the previous one, when anything did.
    pub growth: Option<String>,
    pub workspaces: Vec<WsRow>,
}

impl Chunk {
    /// A channel with nothing behind it yet — what the roster holds before any
    /// answer has come down it.
    pub fn of(channel: Channel) -> Self {
        Self {
            channel,
            ..Self::default()
        }
    }
}
