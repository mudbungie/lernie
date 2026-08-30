//! The command line, as a pure function.
//!
//! `run` takes the arguments and hands back a [`Decided`] — either a
//! [`Verdict`] to say, or a thing to do that needs this process's own
//! environment. It touches no process state: no argv, no environment, no
//! streams, no exit. That is the whole reason `src/main.rs` can be the one file
//! excluded from the coverage floor (`tarpaulin.toml`) without excluding any
//! decision: every decision is here, and every decision is a value a test can
//! read back.

/// Which stream a verdict's text belongs on.
///
/// It is stored rather than derived from the code, and the exception is the
/// reason. For everything this binary says about *itself* the code does say the
/// stream — a refusal is stderr, a `--version` is stdout. But the seat's one
/// product is the engine's **reply stream**, and an engine answering `ok:
/// false` has answered: that is the product, it goes to stdout with the rest of
/// the frames, and only the exit code says no. Deriving the stream from the
/// code would put an answer on stderr because it was a negative one.
/// What this binary says about ITSELF: the version line and the usage.
mod text;

pub use text::{usage, version};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stream {
    /// stdout — this run's product.
    Out,
    /// stderr — this run's diagnosis.
    Err,
}

/// What one invocation decided: an exit code, the text that explains it, and
/// where that text belongs.
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
const REFUSED: u8 = 2;

/// The exit code for a run that was understood and did not finish: no channel,
/// a channel that would not open, an engine that would not answer, or an engine
/// that answered no. One code for the same reason.
const FAILED: u8 = 1;

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
            text: format!("lernie: {what}\n\n{}", usage()),
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

/// What one invocation decided to do.
pub enum Decided {
    /// Say this, and exit. Every flag and every refusal is one of these.
    Say(Verdict),
    /// Describe every channel this box holds. Needs the data root, which is
    /// this process's own environment and so the entry point's to fold.
    Entries,
    /// Send this gesture envelope down the channel it names. Needs the data
    /// root for the same reason.
    ///
    /// **It carries the envelope, not the text it was typed as.** A typed verb
    /// and a hand-written `ask` both arrive here as the same value, so there is
    /// one thing to route and the reading of a caller's JSON — which is a pure
    /// function of what was typed — stays in this pure function where a test
    /// reads its refusal back as a value.
    Ask(serde_json::Value),
}

/// Decide what one invocation does. `args` is argv **without** the program
/// name.
pub fn run(args: Vec<String>) -> Decided {
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    match words.as_slice() {
        ["entries"] => Decided::Entries,
        ["ask", envelope] => ask(envelope),
        // One help, three spellings, because the subject is one.
        ["help" | "--help" | "-h"] => Decided::Say(Verdict::ok(usage())),
        ["help", verb] => Decided::Say(match crate::verbs::help::page(verb) {
            Ok(page) => Verdict::ok(page),
            Err(refusal) => Verdict::refused(refusal),
        }),
        ["--version" | "-V"] => Decided::Say(Verdict::ok(version())),
        ["ask"] => Decided::Say(Verdict::refused(
            "`lernie ask` wants one gesture envelope".to_owned(),
        )),
        [] => Decided::Say(Verdict::refused(
            "nothing to do — the window is not built yet; the typed verbs, \
             `lernie ask` and `lernie entries` are what there is"
                .to_owned(),
        )),
        [word, arguments @ ..] => typed(word, arguments),
    }
}

/// A hand-written envelope. **Read here**, in the pure function, because
/// whether a body is a gesture is decided entirely by what was typed — so it is
/// the caller's typo, it earns the usage, and it costs no connection.
fn ask(text: &str) -> Decided {
    match crate::envelope::parse(text) {
        Ok(envelope) => Decided::Ask(envelope),
        Err(refusal) => Decided::Say(Verdict::refused(refusal)),
    }
}

/// A typed verb, or a first word that is not one.
///
/// The whole argument list still decides — a verb with the wrong number of
/// arguments refuses rather than ignoring the extras — but the refusal it earns
/// names the verb and its grammar, where a word that is no verb at all can only
/// be quoted back.
fn typed(word: &str, arguments: &[&str]) -> Decided {
    let Some(verb) = crate::verbs::find(word) else {
        return Decided::Say(Verdict::refused(format!(
            "unrecognised argument: {}",
            std::iter::once(word)
                .chain(arguments.iter().copied())
                .collect::<Vec<&str>>()
                .join(" ")
        )));
    };
    match verb.envelope(arguments.iter().map(|a| (*a).to_owned()).collect()) {
        Ok(envelope) => Decided::Ask(envelope),
        Err(refusal) => Decided::Say(Verdict::refused(refusal)),
    }
}

#[cfg(test)]
mod tests;
