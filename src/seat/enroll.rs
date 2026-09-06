//! **The enrollment act, from argv** (yog's `docs/REMOTE.md` §8.4): one
//! gesture, and its answer said in every form a box can take it in.
//!
//! # One artifact, drawn twice, and optionally filed
//!
//! The product of an enrollment is the **§8.4 envelope** — one line of compact
//! JSON under `{"yog-enroll":1,…}`. A QR symbol is a picture of that line and
//! nothing else, so this act prints both: the symbol for a camera, the line
//! for a keyboard, and `--into <dir>` for a box that has neither. They are the
//! same bytes and there is one place they are built
//! ([`crate::reply::enrolled::Enrolled::envelope`]).
//!
//! It used to print the picture alone, and say so — *not written down
//! anywhere*. Two components could not be seated through it. A **foot** is by
//! definition a box the operator is not sitting at, with no screen and no
//! camera, so the one component the act exists to provision was the one it
//! could not reach (bl-1554). And the **phone** asks, in its own enrollment
//! screen, for *one line of JSON beginning `{"yog-enroll": 1`* — the exact text
//! the seat was drawing a picture of and discarding (bl-a8fd). Both had the
//! same workaround: spell the envelope by hand through `lernie ask` and
//! reassemble it against §8.4. A seat that makes an operator reimplement the
//! payload contract is not rendering the answer, it is withholding it.
//!
//! **The custody argument does not reach the text**, and that is why this is a
//! change of one line's worth of behaviour rather than a relaxation. What §4.15
//! rules out is a copy *nobody chose* — a cache, a log, a temporary file. The
//! envelope was on the screen either way, drawn as a picture of itself; a
//! photograph of a private key is a private key. What stays true is that this
//! seat keeps nothing of its own accord: with no `--into`, not a byte is
//! written, and the suite still asserts that over the whole tree.
//!
//! # Nothing is written unasked, and that is asserted rather than intended
//!
//! No file, no cache, no log line, no temporary anything. The material lives in
//! this function's locals and dies with them. `enroll::tests` drives the whole
//! act against the stand-in engine over a throwaway root and walks that root
//! afterwards, comparing the tree to what was there before — over the **tree**
//! rather than over the paths this code happens to know about, because a defect
//! here is precisely a path nobody thought of. A stated destination is walked
//! the same way, and what is found there is exactly the four files.

use std::path::Path;

use crate::cli::Verdict;
use crate::qr::Symbol;
use crate::reply::{Read, Reply};

/// The envelope as an entry, where the operator names one.
mod entry;

/// The line under the material. It says the one thing an operator cannot see by
/// looking: that there is no second copy here, so what is on the screen is the
/// only one there will be until another `enroll` is spent.
const KEPT: &str = "the seat keeps no copy — scan the symbol or take the line; they are the same \
     bytes. Enroll again if it is lost";

/// What the seat says when the gesture crossed and no answer came back.
///
/// **`enroll` is the act whose doubt costs the most.** Its product is the one
/// reply this seat never keeps, so a registration that was minted and whose
/// answer was lost leaves a box registered with material that exists nowhere —
/// and `enroll again` (which [`KEPT`] rightly offers when the material WAS
/// said) would mint a second registration over a first nobody can see. So the
/// remedy is REMOTE §3's: read the world first.
const INDOUBT: &str = "the enrollment crossed with no answer, so it is IN DOUBT — a registration may \
     exist whose material is gone. Do not enroll again until you have looked: \
     `lernie ask '{\"op\":\"clients\"}'` says which clients that engine holds";

/// **Enroll a new box**, and say the material every way it can be taken.
pub fn enroll(
    data_root: &Path,
    workspace: &str,
    name: &str,
    grade: &str,
    into: Option<&Path>,
) -> Verdict {
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
        Read::Answer(Reply::Enrolled(material)) => said(&material, into),
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

/// The material: the caption, the picture where one fits, the line always, and
/// where it was filed if a destination was named.
fn said(material: &crate::reply::enrolled::Enrolled, into: Option<&Path>) -> Verdict {
    let envelope = material.envelope();
    // **A symbol that will not fit no longer costs the material.** The ceiling
    // is version 40 at correction level M and REMOTE §8.4 measures the envelope
    // well inside it, so this is a recipe that moved — an RSA key, a longer
    // chain — and saying the size is what makes that legible. But the
    // enrollment is already spent at the engine, so refusing here would burn a
    // name over a picture: the line below is the material, and a camera is one
    // of three ways to take it.
    let picture = match Symbol::encode(envelope.as_bytes()) {
        Ok(symbol) => symbol.block(),
        Err(too_long) => format!("(no symbol: the material will not fit one — {too_long})"),
    };
    let text = format!("{}\n\n{picture}\n{envelope}\n\n{KEPT}", material.caption());
    let Some(dir) = into else {
        return Verdict::ok(text);
    };
    match entry::written(dir, material) {
        Ok(filed) => Verdict::ok(format!("{text}\n{filed}")),
        // The material is above, so nothing was lost — but the operator asked
        // for files and has none, and only the exit code may say so: the text
        // is this run's product and belongs on stdout whichever way it went.
        Err(why) => Verdict::answered(format!("{text}\nnot filed: {why}"), false),
    }
}

#[cfg(test)]
mod tests;
