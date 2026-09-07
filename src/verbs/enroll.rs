//! **The enrollment**: its row, the one optional field its request carries,
//! and the typed door both faces compose it through.
//!
//! Its own file for the reason [`super::rows`]'s doc gives for the other
//! twenty-two declared away from the table — *one subject, one file* — and it
//! is more of a subject than most: the row's reply is the only one this seat
//! renders rather than prints ([`crate::seat::enroll`]), its request is the
//! only one carrying an optional field, and argv's grammar for it is its own
//! file too ([`crate::cli`]). What stays in the table is the enumeration.

use serde_json::Value;

use super::Verb;

/// **The enrollment's row.** Three named strings, so it is a row like any
/// other — but the *reply* is not like any other, and `lernie enroll` therefore
/// has its own arm in [`crate::cli`] rather than printing the reply stream the
/// way every other verb does. What the arm prints is the rendering the operator
/// asked for — the symbol and the line, or, where `--into` named a directory,
/// the receipt alone (bl-768a).
pub const ENROLL: Verb = Verb {
    word: "enroll",
    params: &["workspace", "name", "grade"],
    flags: &[],
    summary: "mint a new box's material and say it as a code, a line and, if asked, four files",
    detail: "The engine mints a leaf on its own CA, seats the client in that \
             workspace, answers the material and shreds the key. What comes \
             back is REMOTE §8.4's envelope — one line of compact JSON under \
             `{\"yog-enroll\":1,…}` — and there are three ways to take those \
             same bytes: a QR symbol for a camera, the line itself for the \
             paste box an android seat offers, and `--into <dir>` for a box \
             with neither, which writes the four files an entry is (`ca.pem`, \
             `client.pem`, `client.key`, `address`) into that directory for \
             you to carry to the machine they are for. `--at <host>:<port>` \
             states the route THAT box will dial, which is a fact about it and \
             not about this engine: unstated is the engine's own address, \
             which is right only for a device that shares this box's view of \
             it. YOU PICK ONE: with no \
             `--into` the symbol and the line are both printed and this seat \
             keeps NOTHING — not a file, not a cache, not a log line; with \
             one, the key goes to the files and never to this terminal, which \
             is the one place the act could not shred it afterwards. \
             `lernie --json enroll …` is for a script: stdout is the envelope \
             alone, on one line, with no symbol and nothing else — and with \
             `--into` it is empty, because the material went to the files. \
             `grade` \
             is `operator` or `foot`. It is refused unless this box's own leaf \
             is operator-grade — the new box says nothing and performs no act, \
             which is why this is not the in-channel bootstrap REMOTE §1.4 \
             forbids.",
};

/// **The §8.4 request's one optional field**, spelled once so the door that
/// states it and the suite that reads it back cannot disagree. Argv spells it
/// [`crate::cli::AT`].
pub const ADDRESS: &str = "address";

/// The enrollment, on the same terms — from a name and a grade the operator
/// chose, and, where they stated one, the route the new box will dial.
///
/// **`address` is a fact about the DEVICE, not about this engine** (REMOTE
/// §8.4 as amended, yog bl-fec6): the box being enrolled is not the box the
/// engine runs on, so the route it reaches that engine by need not be the one
/// the engine wrote for itself — an emulator reaches its host through the
/// emulator's alias, a phone over the LAN, an overlay peer by name. `None` is
/// the engine's own `wire/address`.
pub fn enroll(workspace: String, name: String, grade: String, address: Option<String>) -> Value {
    ENROLL.stating(vec![workspace, name, grade], ADDRESS, address)
}

#[cfg(test)]
mod tests;
