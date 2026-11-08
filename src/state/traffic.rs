//! **What crosses the lock** — a worker's report, and the question set the
//! frame publishes for them ([`standing`]).
//!
//! Split from [`super`] at the design-time budget on the seam the module's own
//! doc already draws: [`super`] is the lock and the two sides' one call each,
//! and this is what they say to each other. The first changes when the
//! threading does; the second when a pane learns to ask something new — and
//! the question set split out again at the same cap when the rails let more
//! than one pane's read stand at once (DESIGN §4.48 item 8).

/// The question set, and which panes' reads are in it.
mod standing;
pub use standing::{Open, Standing};

use serde_json::Value;

use crate::channel::Reach;
use crate::reply::Read;
use crate::ui::Channel;

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
/// this box cannot open is a different sentence from a server that refused:
/// the first is about this box's own files or the far end being down, the
/// second is the server answering. A seat that read them alike would send an
/// operator to check a certificate over a workspace name they mistyped.
///
/// The fourth arm is that division taken one step further (bl-3969): what a
/// failed leg means depends on whether the gesture was an ACT, so the poster
/// reports one and the two read workers report the other.
#[derive(Debug, Clone)]
pub enum Said {
    /// **A read about ONE conversation, stamped with which** — the follow
    /// lane's accumulation so far, and the selection's own standing reads
    /// (bl-19b8: a transcript read in flight when the selection moved was
    /// filed under whatever was selected when it landed).
    ///
    /// The stamp is what makes a stale tail impossible rather than unlikely.
    /// The server was asked about a conversation and answers about that one, so
    /// only this end knows the focus has moved — and only the FRAME knows what
    /// it is looking at right now. So the lane says what its frames are about
    /// and the frame decides whether they are still wanted, which is a pure
    /// comparison at the one place that holds the answer, rather than a poll
    /// racing the socket at the other.
    ///
    /// **Already read**, unlike [`Receipt`](Self::Receipt): a follow frame is an
    /// append (REMOTE §5.5), so it has to be absorbed onto the read's own fold
    /// before it means anything, and the lane is where a read begins and ends.
    /// `op` is what was asked, the one fact a refusal cannot carry for
    /// itself (bl-6dfd): a refused `agent` read says the subject is gone.
    About {
        conversation: String,
        op: String,
        read: Read,
    },
    /// **A sign-in lane's fold so far, stamped with the row it is about**
    /// (REMOTE §8.3, bl-e3c5) — [`About`](Self::About)'s shape one noun over,
    /// and stamped for the identical reason: the server was asked about one
    /// provider and answers about that one, so only the FRAME knows whether the
    /// operator is still following it. A run started on a second row replaces
    /// the first upstream, and an unstamped frame would paint one run's lines
    /// under the other's name.
    Signin { provider: String, read: Read },
    /// **One provider row's `models` answer, stamped with the row asked**
    /// (bl-b7d9). The reply names no provider, and the tuning pane asks every
    /// row its roles run on in one pass, so only the asker knows whose it is.
    Offering { provider: String, read: Read },
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
    Acted {
        op: String,
        reach: Reach,
        /// **The words the gesture carried**, where it carried any
        /// (`crate::envelope::said`). They ride with the act for `op`'s own
        /// reason: an act that provably never left this box is one the
        /// operator has to type again, and the seat should not be the reason
        /// they do. A field on the model instead would be a second copy of
        /// something already in flight, and nothing could say which act's copy
        /// had come back.
        said: Option<String>,
    },
    /// **One reply frame, exactly as it crossed**, stamped with the op it
    /// answers (bl-b180).
    ///
    /// A refusal wears no `kind`, so a frame alone cannot say which exchange
    /// it closes — and the one thing this window holds across two acts, a
    /// start, has to know whether the sentence that came back is *its* refusal
    /// or somebody else's. The poster is the one place that still knows, for
    /// [`Acted`](Self::Acted)'s reason exactly, so it says so on the way in
    /// rather than the model guessing from timing. **A standing read's frame
    /// is stamped too** (bl-693d), and there is no bare frame left: the
    /// roster read's refusal is a fact about the channel — this client is
    /// registered nowhere there — and only its op can say so.
    Receipt { op: String, frame: Value },
}
