//! The command line, as a pure function.
//!
//! `run` takes the arguments and hands back a [`Decided`] — either a
//! [`Verdict`] to say, or a thing to do that needs this process's own
//! environment. It touches no process state: no argv, no environment, no
//! streams, no exit. That is the whole reason `src/main.rs` can be the one file
//! excluded from the coverage floor (`tarpaulin.toml`) without excluding any
//! decision: every decision is here, and every decision is a value a test can
//! read back.

/// What this binary says about ITSELF: the version line and the usage.
mod text;
/// What an invocation says, and with what exit code.
mod verdict;

pub use text::{usage, version};
pub use verdict::{Stream, Verdict};

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
    Start { address: String, goal: String },
    /// **Enroll a new box** in that workspace, under that name, at that grade
    /// (REMOTE §8.4): one gesture, and a symbol printed instead of its answer.
    ///
    /// It has an arm of its own — rather than riding [`Ask`](Self::Ask) like
    /// every other typed verb — because the reply carries a private key for a
    /// box that does not exist yet, and the reply stream's destination is a
    /// terminal's scrollback. What the act prints is the picture; see
    /// [`crate::seat::enroll`].
    Enroll {
        workspace: String,
        name: String,
        grade: String,
    },
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
        // The composite, and its arity is exact for [`crate::verbs`]'s own
        // reason: argv quotes, so a goal is one argument, and an unquoted tail
        // refuses rather than being silently joined.
        ["start", address, goal] => Decided::Start {
            address: (*address).to_owned(),
            goal: (*goal).to_owned(),
        },
        // Ahead of the typed table, and only because of what the answer
        // carries: the row is the same row, and the envelope is built from it.
        ["enroll", workspace, name, grade] => Decided::Enroll {
            workspace: (*workspace).to_owned(),
            name: (*name).to_owned(),
            grade: (*grade).to_owned(),
        },
        ["start", ..] => Decided::Say(Verdict::refused(
            "`lernie start` takes a workspace and a goal — usage:              lernie start <workspace> <goal>"
                .to_owned(),
        )),
        // The bare invocation is the window, because a seat is a window. Every
        // other spelling is a way of reaching one gesture without one.
        [] => Decided::Window,
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
