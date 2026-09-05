//! **Reading one frame** — the dispatch off `kind`, and the two rungs that are
//! the envelope's own rather than a type's.
//!
//! Split from [`super`] at the design-time budget on the seam the module
//! already has: [`super`] is *what a reply is* — the roster of kinds and the
//! policy every reader obeys — and this is *how one frame becomes one*. The
//! per-kind readings are each beside their own type, so what is left here is
//! exactly the part that belongs to no type: the envelope's shape, the
//! discriminant, and the refusal that wears no kind at all.

use serde_json::Value;

use super::{
    ERROR, KIND, OK, Outcome, Read, Reply, convs, enrolled, fields, files, help, login, ops,
    providers, queue, roles, roster, search, start, steps, stream, transcript,
};

/// The kind token each arm answers to. Its type's own file holds the rest, so
/// the tokens live where the reading does and a spelling cannot drift from it;
/// these two answer to no type of their own.
const OUTCOME: &str = "outcome";
const NUDGED: &str = "nudged";
/// The flag's receipt. It answers to no type for [`NUDGED`]'s own reason:
/// what it changed arrives on the next queue, so there is nothing to read.
const FLAGGED: &str = "flagged";

/// **Read one reply frame.** Total: every input answers one of [`Read`]'s
/// three arms, and none of them is a panic.
pub fn read(frame: &Value) -> Read {
    match decode(frame) {
        Ok(read) => read,
        Err(why) => Read::Unreadable(why),
    }
}

/// The reading proper, with rung 1's refusals as the `Err`. Split from
/// [`read`] so every `?` in it lands on one arm rather than at each site.
fn decode(frame: &Value) -> Result<Read, String> {
    let obj = frame
        .as_object()
        .ok_or("reply: not a JSON object".to_owned())?;
    let Some(kind) = obj.get(KIND) else {
        return refusal(fields::flag(obj, OK)?, fields::text(obj, ERROR));
    };
    let kind = kind
        .as_str()
        .ok_or_else(|| format!("reply: non-string field {KIND:?}"))?;
    let reply = match kind {
        OUTCOME => Reply::Outcome(Outcome {
            exit: fields::exit(obj)?,
            stdout: fields::text(obj, "stdout")?,
            stderr: fields::text(obj, "stderr")?,
        }),
        NUDGED => Reply::Nudged,
        roster::KIND => Reply::Workspaces(roster::workspaces(obj)?),
        convs::KIND => Reply::Conversations(fields::rows(obj, convs::row)?),
        roles::KIND => Reply::Roles(fields::rows(obj, roles::row)?),
        queue::KIND => Reply::Attention(fields::rows(obj, queue::row)?),
        FLAGGED => Reply::Flagged,
        transcript::KIND => Reply::Transcript(transcript::transcript(obj)?),
        steps::KIND => Reply::Steps(steps::steps(obj)?),
        files::KIND => Reply::Files(files::files(obj)?),
        help::KIND => Reply::Help(fields::rows(obj, help::row)?),
        search::KIND => Reply::Found(search::found(obj)?),
        ops::KIND => Reply::Ops(fields::rows(obj, ops::row)?),
        providers::KIND => Reply::Providers(fields::rows(obj, providers::row)?),
        providers::MODELS => Reply::Models(fields::rows(obj, providers::offered)?),
        login::KIND => Reply::Login(login::signin(obj)?),
        stream::KIND => Reply::Follow(stream::follow(obj)?),
        enrolled::KIND => Reply::Enrolled(enrolled::enrolled(obj)?),
        start::PREPARED => Reply::Prepared(start::prepared(obj)?),
        start::STARTED => Reply::Started {
            conversation: start::started(obj)?,
        },
        // Rung 2. The kind is named because naming it is the whole remedy: an
        // operator reading it knows which end is behind, exactly as the
        // version preface's mismatch names both numbers.
        other => {
            return Err(format!(
                "reply: this seat cannot paint a {other:?} answer — the engine \
                 speaks a kind this build does not; upgrade the seat"
            ));
        }
    };
    Ok(Read::Answer(reply))
}

/// The kind-less envelope, and nothing else may wear that shape: a refusal is
/// `{"ok": false, "error": …}` and an object claiming `ok` with no kind is an
/// answer that failed to say what it answers.
fn refusal(ok: bool, error: Result<String, String>) -> Result<Read, String> {
    if ok {
        return Err(format!("reply: an answer with no {KIND:?}"));
    }
    error.map(Read::Refusal)
}
