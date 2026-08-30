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
    Ask(String),
}

/// The crate's name and version, as the `--version` line.
///
/// **The version is the fence** (yog's `docs/REMOTE.md` §12): `lernie` through
/// 0.0.x was the agent-loop engine, which continues as `litany`; `lernie` at
/// 0.1.0 and above is this seat. So the number printed here is not decoration —
/// it is the one rule that says which program the caller is holding.
pub fn version() -> String {
    format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
}

/// The usage text. It states the fence before it states what to type, because
/// the name has two eras in the published record and a caller who typed
/// `lernie` may be holding the other one.
pub fn usage() -> String {
    format!(
        "{}

lernie is the seat: the operator's face on a yog server. It dials in over
mTLS, asks and acts, and paints what comes back. It holds no world, runs no
agent and executes nothing.

THE NAME HAS TWO ERAS. lernie through 0.0.x was the agent-loop engine, which
continues under the name litany. lernie 0.1.0 and above is this seat. The
version is the only rule that separates them.

usage: lernie entries
       lernie ask <envelope>
       lernie [--version | --help]

  entries         describe every channel this box holds, without dialling any
                  of them: its own engine, then one row per workspace held
                  elsewhere, each with its address or the reason it has none.
  ask <envelope>  send one gesture envelope — the JSON object the boundary
                  already carries, `op` the discriminant — down the channel
                  its workspace names, and print the reply stream, one
                  envelope per line. Exit 0 when the last reply says ok.
  -V, --version   print the name and version
  -h, --help      print this

What it reads, all of it provisioned by hand and none of it ever written by
lernie: this box's own channel at <data root>/wire/, and one channel per
directory under <data root>/wire/workspaces/. The data root is
$XDG_DATA_HOME/lernie, or $HOME/.local/share/lernie.

See docs/DESIGN.md for the role and the module map, and yog's docs/REMOTE.md
for the protocol lernie implements against.",
        version()
    )
}

/// Decide what one invocation does. `args` is argv **without** the program
/// name.
pub fn run(args: Vec<String>) -> Decided {
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    match words.as_slice() {
        ["entries"] => Decided::Entries,
        ["ask", envelope] => Decided::Ask((*envelope).to_owned()),
        ["--version" | "-V"] => Decided::Say(Verdict::ok(version())),
        ["--help" | "-h"] => Decided::Say(Verdict::ok(usage())),
        ["ask"] => Decided::Say(Verdict::refused(
            "`lernie ask` wants one gesture envelope".to_owned(),
        )),
        [] => Decided::Say(Verdict::refused(
            "nothing to do — the window is not built yet; `lernie entries` and \
             `lernie ask` are the verbs"
                .to_owned(),
        )),
        other => Decided::Say(Verdict::refused(format!(
            "unrecognised argument: {}",
            other.join(" ")
        ))),
    }
}

#[cfg(test)]
mod tests;
