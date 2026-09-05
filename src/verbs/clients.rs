//! **The machines registered in one workspace** (yog's `docs/REMOTE.md` §5,
//! §5.1) — the one op on the tool-host surface a seat is owed.
//!
//! Its own module rather than a row among [`super::rows`]'s reads, on that
//! module's own seam: what is there is the enumeration a seat had first, and
//! this is a subject — *who may execute for this workspace, and what they say
//! they can do*. The rest of that subject is not a seat's at all, and saying so
//! is the reason this file exists rather than one more `pub const`.
//!
//! # Four of the family's five ops are a MACHINE's, and none of them is here
//!
//! `advertise`, `invocations`, `complete`, `invoke` and `capture` are all
//! classed `machine` by the engine's own help table — the field yog's PARITY §7
//! roster is generated from — and a seat owes a control for none of them. Two
//! are worth naming, because they are the two an operator might reasonably
//! expect on this pane:
//!
//! - **`invocations` cannot be READ by a seat, even to look.** It is
//!   follow-class and it *drains*: it parks until the engine's hold expires
//!   and answers the queue addressed to *the certificate this connection
//!   presented* (REMOTE §5.3). So a seat asking it would learn nothing about
//!   any foot — its own queue is empty by construction — while a box whose
//!   seat and foot share one identity would have its work handed to the window
//!   and never completed, and its foot's own read refused in band as a second
//!   reader (§5.1). The invocation rows a foot is handed are its own; what an
//!   operator sees of one is the queue row's parked call (§4.19) and the step
//!   drill-in's record (bl-3257).
//! - **`advertise` is the machine's statement about itself.** A seat that
//!   presented a set would be claiming to be a tool host, and the set it
//!   replaced would be a real machine's.
//!
//! `corpus/unreadable/invocations.json` therefore stays where it is, and it is
//! not an unpainted pane waiting on a ball: it is a shape addressed to a
//! different kind of client.

use serde_json::Value;

use super::Verb;

/// **The listing**: every client registered in one workspace.
pub const CLIENTS: Verb = Verb {
    word: "clients",
    params: &["workspace"],
    summary: "the machines registered in this workspace, who is connected, and \
              what they offer",
    detail: "One row per client registered in the named workspace: its name, \
             whether it holds a live connection right now, and the tools it \
             has advertised — each with whether that box consents to run it at \
             a working directory the invocation names. Presence is read at the \
             moment you ask and is true only then; a tool host holds its \
             connection while it waits for work, so a busy machine looks \
             absent. What each client advertises was written when it last \
             presented its set and stands whether or not it is connected. A \
             machine is registered by an operator's own act on the server, \
             never over this wire.",
};

/// **The machines this workspace holds**, asked of the wall `workspace` names.
pub fn clients(workspace: String) -> Value {
    CLIENTS.built(vec![workspace])
}
