//! **The enrollment act, from argv** (yog's `docs/REMOTE.md` §8.4): one
//! gesture, and a symbol printed instead of the answer.
//!
//! # It is the one verb whose reply is not the product
//!
//! Every other typed verb hands its reply stream straight to stdout — that is
//! what a seat is, and [`crate::seat::ask`] is the whole of it. This one must
//! not: the `enrolled` answer carries a **private key for a box that does not
//! exist yet**, and stdout is a terminal's scrollback, a shell's history file
//! and whatever the operator piped it into. So `enroll` has an arm of its own
//! all the way up to [`crate::cli`], and what it prints is the picture.
//!
//! **`lernie ask` is still the escape hatch and still prints the raw frame**,
//! which is correct: an operator who spells the envelope by hand has asked for
//! the stream, and nothing here is a security boundary — the seat holds no
//! secret from its own operator. What this arm buys is that the *ordinary* path
//! does not leave material somewhere nobody chose to put it.
//!
//! # Nothing is written, and that is asserted rather than intended
//!
//! No file, no cache, no log line, no temporary anything. The material lives in
//! this function's locals and dies with them. `enroll::tests` drives the whole
//! act against the stand-in engine over a throwaway root and walks that root
//! afterwards, comparing the tree to what was there before — over the **tree**
//! rather than over the paths this code happens to know about, because a defect
//! here is precisely a path nobody thought of.

use std::path::Path;

use crate::cli::Verdict;
use crate::qr::Symbol;
use crate::reply::{Read, Reply};

/// The line under the symbol. It says the one thing an operator cannot see by
/// looking: that there is no second copy, so this picture is the only one there
/// will be until another `enroll` is spent.
const KEPT: &str = "not written down anywhere — scan it now, or enroll again";

/// What the seat says when the gesture crossed and no answer came back.
///
/// **`enroll` is the act whose doubt costs the most.** Its product is the one
/// reply this seat never keeps, so a registration that was minted and whose
/// answer was lost leaves a box registered with material that exists nowhere —
/// and `enroll again` (which [`KEPT`] rightly offers when the picture WAS drawn)
/// would mint a second registration over a first nobody can see. So the remedy
/// is REMOTE §3's: read the world first.
const INDOUBT: &str = "the enrollment crossed with no answer, so it is IN DOUBT — a registration may \
     exist whose material is gone. Do not enroll again until you have looked: \
     `lernie ask '{\"op\":\"clients\"}'` says which clients that engine holds";

/// **Enroll a new box**, and print the symbol its material rides in.
pub fn enroll(data_root: &Path, workspace: &str, name: &str, grade: &str) -> Verdict {
    let gesture = crate::verbs::enroll(workspace.to_owned(), name.to_owned(), grade.to_owned());
    let stream = match crate::seat::sent(data_root, &gesture) {
        Ok(stream) => stream,
        Err(reach) if reach.crossed() => {
            return Verdict::failed(format!("{INDOUBT}: {}", reach.said()));
        }
        Err(reach) => return Verdict::failed(reach.said()),
    };
    let Some(frame) = stream.last() else {
        return Verdict::failed("the engine answered nothing at all".to_owned());
    };
    match crate::reply::read(frame) {
        Read::Answer(Reply::Enrolled(material)) => drawn(&material),
        // A refusal is the engine answering, so it is this run's product and
        // goes to stdout with the exit code saying no — the same rule
        // [`crate::seat::ask`] keeps. It carries no material to withhold.
        Read::Refusal(said) => Verdict::answered(said, false),
        Read::Unreadable(why) => Verdict::failed(why),
        // A well-formed answer of the wrong kind. It is not unreadable — this
        // seat read it — so saying "cannot read" would send an operator to
        // upgrade something that is fine.
        Read::Answer(_) => Verdict::failed(format!(
            "`enroll` was answered with something else entirely; the engine at \
             {workspace:?} did not mint anything"
        )),
    }
}

/// The material as a picture, and the two lines around it.
fn drawn(material: &crate::reply::enrolled::Enrolled) -> Verdict {
    let envelope = material.envelope();
    match Symbol::encode(envelope.as_bytes()) {
        Ok(symbol) => Verdict::ok(format!(
            "{}\n\n{}\n{KEPT}",
            material.caption(),
            symbol.block()
        )),
        // The ceiling is version 40 at correction level M and REMOTE §8.4
        // measures the envelope well inside it, so this arm is a recipe that
        // moved — an RSA key, a longer chain — rather than a symbol that could
        // have been drawn smaller. Saying the size is what makes that legible.
        Err(too_long) => Verdict::failed(format!(
            "the engine's material will not fit one symbol: {too_long}"
        )),
    }
}

#[cfg(test)]
mod tests;
