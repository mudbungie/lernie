//! **What one invocation decided to do** — the value [`super::run`] hands
//! back, and the whole of what `src/main.rs` acts on.
//!
//! Split from [`super`] at the 300-line cap on the seam that module already
//! draws twice ([`super::verdict`], [`super::text`]): [`super`] is the
//! DECIDING — the match over argv and the three readings it makes — and this
//! is the shape of what it decides. They change for different reasons: a word
//! added moves the match, a KIND of act added moves the enum, and the second
//! is much rarer.

use super::Verdict;
use crate::render::Form;

/// What one invocation decided to do.
#[derive(Debug)]
pub enum Decided {
    /// Say this, and exit. Every flag and every refusal is one of these.
    Say(Verdict),
    /// Describe every channel this box holds. Needs the data root, which is
    /// this process's own environment and so the entry point's to fold.
    Entries,
    /// **Open the window**, which is what a seat is for. It needs the data root
    /// and a native event loop, both of which are the entry point's — so it
    /// carries nothing, exactly as [`Entries`](Self::Entries) does.
    Window,
    /// **Begin a conversation** in that workspace, with that goal: the §8.1
    /// start family's two acts, spelled as one word. It is a serialization and
    /// not a gesture — what crosses is `prepare` and then `prompt`, the
    /// boundary's own envelopes — and it is one word because the thing between
    /// them is a local: the staged body, held while the second act is composed.
    ///
    /// It carries the two words rather than an envelope, because there are two
    /// envelopes and the second cannot be built until the first is answered.
    Start {
        address: String,
        goal: String,
        /// **The work target, when one was named** — the §3.4 path rung, and
        /// `None` for the bare one. The rung is said outright rather than
        /// inferred, so this `Option` IS the rung and there is no second word
        /// for the operator to keep in agreement with it.
        dir: Option<String>,
        /// **The role the conversation is born on, when one was asked for**
        /// (REMOTE §9.21, PROTOCOL 18). `None` leaves the body's own value,
        /// which is the `null` `prepare` answers and litany reads as `worker`
        /// — so a start that names no role is the start this seat sent before
        /// the field existed. It is stated on the FIRE and not on the stage,
        /// which is why it is a word here and not a rung.
        role: Option<String>,
        /// Which form the two reply streams print in.
        form: Form,
    },
    /// **Register this box** with the offer at that URL (REMOTE §15.1 steps 2
    /// to 4): a key made here, the offer claimed over HTTPS, the entry filed,
    /// the confirmation word shown, and the wire dialled once. Not a gesture —
    /// nothing crosses the boundary until the entry exists — so it carries the
    /// URL and the entry point adds the two things that are the process's: the
    /// data root and the public web's trust roots ([`crate::seat::register`]).
    Register(String),
    /// Send this gesture envelope down the channel it names. Needs the data
    /// root for the same reason.
    ///
    /// **It carries the envelope, not the text it was typed as.** A typed verb
    /// and a hand-written `ask` both arrive here as the same value, so there is
    /// one thing to route and the reading of a caller's JSON — which is a pure
    /// function of what was typed — stays in this pure function where a test
    /// reads its refusal back as a value.
    /// **Watch one conversation until it rests** (bl-3dca, bl-f076) — the
    /// §5.1 follow read, held across the server's step boundaries.
    ///
    /// It is a serialization and not a gesture, exactly as
    /// [`Start`](Self::Start) is: what crosses is `agent` and `follow`, the
    /// boundary's own envelopes, asked as many times as it takes. One word,
    /// because the server ends a read at the step and an operator's question
    /// is about the TURN — and about the whole of the work after it.
    Follow {
        workspace: String,
        agent: String,
        form: Form,
    },
    /// **Give a role this model** (bl-1e5a), having first asked the provider
    /// row what it offers. A serialization of two reads-and-a-write, not a
    /// gesture: `models` then `model`, the boundary's own envelopes.
    Model {
        workspace: String,
        role: String,
        provider: String,
        model: String,
        form: Form,
    },
    Ask(serde_json::Value, Form),
    /// **Ask this gesture of every channel this box holds**, and answer with
    /// the union stamped with where each answer came from (bl-0d54).
    ///
    /// It is the same envelope [`Ask`](Self::Ask) carries and there is no
    /// second spelling of it — what differs is how many channels it is asked
    /// of. A verb with no `workspace` parameter has no way to name one
    /// ([`crate::verbs::Verb::addresses_a_workspace`]), so its subject is all
    /// of them: the window's roster has always been that union, and the CLI's
    /// shorthand for the same question answered one channel and said nothing
    /// about the rest.
    ///
    /// `lernie ask` stays the raw door and is never fanned: it is the escape
    /// hatch for one channel, and `{"op":"workspaces","workspace":"<leaf>"}` is
    /// how an operator asks exactly one of them.
    Fanned(serde_json::Value, Form),
    /// **Ask this workspace-less ACT of one channel** (bl-1bb9): the one `on`
    /// names, or — where it names none — the one channel this box holds, and
    /// a refusal naming them all on a box holding more. A write a person types
    /// once means one server, where [`Fanned`](Self::Fanned) is a read whose
    /// subject is every one ([`crate::seat::aimed`]).
    Aimed {
        on: Option<String>,
        envelope: serde_json::Value,
        form: Form,
    },
}
