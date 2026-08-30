//! **What an invocation says, and with what exit code.**
//!
//! Split from [`super`] at the design-time budget on the seam the two already
//! have: [`super`] is what an invocation *decides*, and this is the shape of
//! what it hands back. They change for different reasons — a verb added moves
//! the match above, an outcome class added moves the constructors below — which
//! is the test that a seam is real.

/// Which stream a verdict's text belongs on.
///
/// It is stored rather than derived from the code, and the exception is the
/// reason. For everything this binary says about *itself* the code does say the
/// stream — a refusal is stderr, a `--version` is stdout. But the seat's one
/// product is the engine's **reply stream**, and an engine answering `ok:
/// false` has answered: that is the product, it goes to stdout with the rest of
/// the frames, and only the exit code says no. Deriving the stream from the
/// code would put an answer on stderr because it was a negative one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stream {
    /// stdout — this run's product.
    Out,
    /// stderr — this run's diagnosis.
    Err,
}

/// What one invocation decided: an exit code, the text that explains it, and
/// where that text belongs.
#[derive(Debug)]
pub struct Verdict {
    /// The process exit code. `0` is the only success.
    pub code: u8,
    /// Everything this run has to say, without a trailing newline.
    pub text: String,
    /// Which stream [`text`](Self::text) goes to.
    pub stream: Stream,
}

/// The exit code for every refusal: bad usage, or a body that is not a gesture.
/// One code, because these are all the same kind of event — "that is not
/// something this binary can act on" — and a taxonomy of exit codes would be a
/// promise to keep them stable.
pub(super) const REFUSED: u8 = 2;

/// The exit code for a run that was understood and did not finish: no channel,
/// a channel that would not open, an engine that would not answer, or an engine
/// that answered no. One code for the same reason.
pub(super) const FAILED: u8 = 1;

impl Verdict {
    /// A successful run and what it printed.
    pub fn ok(text: String) -> Self {
        Self {
            code: 0,
            text,
            stream: Stream::Out,
        }
    }

    /// **The engine's answer**, whichever way it went: the reply stream is this
    /// seat's product, so it goes to stdout, and `ok` is the exit code alone.
    pub fn answered(text: String, ok: bool) -> Self {
        Self {
            code: if ok { 0 } else { FAILED },
            text,
            stream: Stream::Out,
        }
    }

    /// A refusal, from the sentence naming what was refused.
    ///
    /// The prefix and the usage are appended HERE rather than at each call
    /// site, so "a refusal always says what it refused *and* what the caller
    /// could have typed instead" is structural rather than remembered: a
    /// refusal added later cannot forget it. A bare non-zero exit teaches
    /// nobody anything.
    pub fn refused(what: String) -> Self {
        Self {
            code: REFUSED,
            text: format!("lernie: {what}\n\n{}", super::usage()),
            stream: Stream::Err,
        }
    }

    /// A run that did what it was asked and could not finish it.
    ///
    /// It carries **no usage**, and that is the difference from a refusal: a
    /// refusal is about what the caller typed, so the alternatives are the
    /// useful thing to say next; a failure is about this box or the far end,
    /// where a usage line is noise in front of the sentence that matters.
    pub fn failed(what: String) -> Self {
        Self {
            code: FAILED,
            text: format!("lernie: {what}"),
            stream: Stream::Err,
        }
    }
}
