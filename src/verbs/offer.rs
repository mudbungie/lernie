//! **The offer's four words** (yog's `docs/REMOTE.md` §15.1; DESIGN §4.51):
//! mint one, list the pending ones, approve one, decline one.
//!
//! One subject, one file: a new device joins a workspace by an offer an
//! operator-grade seat mints, and the same seat approves it by the
//! confirmation word once the device has dialled. All four are plain rows —
//! every parameter a named string — so the command line's table spells them,
//! the window composes them through the typed doors below, and the corpus
//! round-trips them. They replace `enroll`, whose reply carried a private
//! key; nothing here carries one.

use serde_json::Value;

use super::Verb;

/// The two grades an offer may state, as the wire spells them.
pub const GRADES: [&str; 2] = ["operator", "foot"];

/// Mint an offer.
pub const OFFER: Verb = Verb {
    word: "offer",
    params: &["workspace", "name", "grade"],
    flags: &[],
    summary: "offer a new device a place in this workspace: one URL, good once, for ten minutes",
    detail: "The server records a pending offer for a client of that name \
             and grade, and answers a one-time registration URL with the \
             moment it expires. That URL is the whole of what the new device \
             needs: it is printed as text and as a QR symbol, and it carries \
             no key and no certificate — the device makes its own key and \
             sends only the public half when it claims the URL (`lernie \
             register <url>` on a lernie seat). `grade` is `operator` or \
             `foot`. The server refuses a name anything was ever issued \
             under; minting again under a name whose offer is still open \
             replaces the code. THEN: the device shows a six-character \
             confirmation word. `lernie offers <workspace>` shows the same \
             word beside the offer once the device has claimed it; `lernie \
             approve <workspace> <name>` seats it. Operator grade only.",
};

/// List a workspace's pending offers.
pub const OFFERS: Verb = Verb {
    word: "offers",
    params: &["workspace"],
    flags: &[],
    summary: "this workspace's pending offers, each with its state and confirmation word",
    detail: "Every offer not yet approved or declined: its name, its grade, \
             its state and when its code expires. `open` is a code nobody has \
             claimed; `claimed` is a device that holds the certificate and \
             has not dialled; `proved` is a device that has dialled with it, \
             which a copy of the URL never could. A claimed or proved row \
             carries the confirmation word — the six characters the device \
             is showing. Only a proved offer can be approved.",
};

/// Seat a proved offer's device.
pub const APPROVE: Verb = Verb {
    word: "approve",
    params: &["workspace", "name"],
    flags: &[],
    summary: "seat a proved offer's device in this workspace",
    detail: "Registers the offered client in the workspace, which is the \
             moment its certificate starts to mean anything: until then the \
             server refuses it everything. Refused unless the offer is \
             proved. Read the word off the new device and approve the row \
             that shows the same one — that comparison is the whole of what \
             stops an impostor who claimed the URL first.",
};

/// Void an offer.
pub const DECLINE: Verb = Verb {
    word: "decline",
    params: &["workspace", "name"],
    flags: &[],
    summary: "void an offer; one already claimed keeps its name spent",
    detail: "An open offer is forgotten and its name is free again. An offer \
             somebody already claimed has issued a certificate that cannot be \
             recalled, so declining it keeps that certificate refused for \
             good and spends the name: offer the device again under another \
             one. Decline a row whose word matches nothing in front of you.",
};

/// The window's door onto [`OFFER`].
pub fn offer(workspace: String, name: String, grade: String) -> Value {
    OFFER.built(vec![workspace, name, grade], &[])
}

/// The window's door onto [`OFFERS`].
pub fn offers(workspace: String) -> Value {
    OFFERS.built(vec![workspace], &[])
}

/// The window's door onto [`APPROVE`].
pub fn approve(workspace: String, name: String) -> Value {
    APPROVE.built(vec![workspace, name], &[])
}

/// The window's door onto [`DECLINE`].
pub fn decline(workspace: String, name: String) -> Value {
    DECLINE.built(vec![workspace, name], &[])
}
