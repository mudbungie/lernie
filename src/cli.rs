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

use crate::render::Form;

/// **The one word this binary reads about its own OUTPUT** rather than about a
/// gesture: `--json` prints the frames exactly as they crossed, where the
/// default renders them for a person ([`crate::render`]).
///
/// **It is read off the FRONT and nowhere else**, and the position is the
/// whole of what makes it unambiguous. Every gesture parameter on this surface
/// is a verbatim string — a goal, a message's content, a ball's title — so a
/// flag scanned out of the middle or the tail would silently eat an operator's
/// own text, and `lernie message w a --json` would stop being a way to send
/// those seven characters. Leading, everything after it is the gesture, byte
/// for byte. Typing it at the end is the mistake this costs, and [`misplaced`]
/// is what that mistake earns.
pub const JSON: &str = "--json";

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
    Start {
        address: String,
        goal: String,
        /// **The work target, when one was named** — the §3.4 path rung, and
        /// `None` for the bare one. The rung is said outright rather than
        /// inferred, so this `Option` IS the rung and there is no second word
        /// for the operator to keep in agreement with it.
        dir: Option<String>,
        /// Which form the two reply streams print in.
        form: Form,
    },
    /// **Enroll a new box** in that workspace, under that name, at that grade
    /// (REMOTE §8.4): one gesture, and its answer rendered rather than handed
    /// on as a frame.
    ///
    /// It has an arm of its own — rather than riding [`Ask`](Self::Ask) like
    /// every other typed verb — because the answer is not a reply stream: it is
    /// the §8.4 **envelope**, which is a different object with two fields
    /// fewer, and it is drawn as a symbol beside the line it encodes. See
    /// [`crate::seat::enroll`].
    ///
    /// `into` is the destination the operator named, and it is `None` on the
    /// ordinary path — the seat writes nothing it was not asked to write.
    Enroll {
        workspace: String,
        name: String,
        grade: String,
        into: Option<String>,
    },
    /// Send this gesture envelope down the channel it names. Needs the data
    /// root for the same reason.
    ///
    /// **It carries the envelope, not the text it was typed as.** A typed verb
    /// and a hand-written `ask` both arrive here as the same value, so there is
    /// one thing to route and the reading of a caller's JSON — which is a pure
    /// function of what was typed — stays in this pure function where a test
    /// reads its refusal back as a value.
    Ask(serde_json::Value, Form),
    /// **Ask this gesture of every channel this box holds**, and answer with
    /// the union stamped with where each answer came from (bl-0d54).
    ///
    /// It is the same envelope [`Ask`](Self::Ask) carries and there is no
    /// second spelling of it — what differs is how many channels it is asked
    /// of. A verb with no `workspace` parameter has no way to name one
    /// ([`crate::verbs::Verb::addresses_a_workspace`]), so its subject is all
    /// of them: the window's roster has always been that union, and the CLI's
    /// shorthand for the same question answered one channel and said nothing
    /// about the rest.
    ///
    /// `lernie ask` stays the raw door and is never fanned: it is the escape
    /// hatch for one channel, and `{"op":"workspaces","workspace":"<leaf>"}` is
    /// how an operator asks exactly one of them.
    Fanned(serde_json::Value, Form),
}

/// Decide what one invocation does. `args` is argv **without** the program
/// name.
pub fn run(args: Vec<String>) -> Decided {
    let (form, args) = form(args);
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    match words.as_slice() {
        ["entries"] => Decided::Entries,
        ["ask", envelope] => ask(envelope, form),
        // One help, three spellings, because the subject is one.
        ["help" | "--help" | "-h"] => Decided::Say(Verdict::ok(usage())),
        ["help", verb] => Decided::Say(match crate::verbs::help::page(verb) {
            Ok(page) => Verdict::ok(page),
            Err(refusal) => Verdict::refused(refusal),
        }),
        ["--version" | "-V"] => Decided::Say(Verdict::ok(version())),
        // The composite, and its arity is exact for [`crate::verbs`]'s own
        // reason: argv quotes, so a goal is one argument, and an unquoted tail
        // refuses rather than being silently joined. **Every other arity falls
        // through to `typed`**, where `start`'s own door row states the usage
        // (bl-60e6): the hand-written refusal that used to stand here carried
        // fourteen literal spaces inside its usage line, a wrapping artifact
        // from when the sentence was laid out over two rows in the source, and
        // a stored usage line is the second fact `crate::verbs` exists not to
        // keep.
        ["start", address, goal] => started(address, goal, None, form),
        ["start", address, goal, dir] => started(address, goal, Some(dir), form),
        // Ahead of the typed table, and only because of what the answer
        // carries: the row is the same row, and the envelope is built from it.
        ["enroll", workspace, name, grade, tail @ ..] => enroll(workspace, name, grade, tail),
        // The bare invocation is the window, because a seat is a window. Every
        // other spelling is a way of reaching one gesture without one.
        [] => Decided::Window,
        [word, arguments @ ..] => typed(word, arguments, form),
    }
}

/// **The composite start**, with or without the work target that makes it the
/// §3.4 path rung rather than the bare one (bl-4371).
fn started(address: &str, goal: &str, dir: Option<&str>, form: Form) -> Decided {
    Decided::Start {
        address: address.to_owned(),
        goal: goal.to_owned(),
        dir: dir.map(str::to_owned),
        form,
    }
}

/// **Read the output form off the front**, and hand back the gesture that is
/// left. See [`JSON`] for why the front and only the front.
fn form(args: Vec<String>) -> (Form, Vec<String>) {
    match args.split_first() {
        Some((first, rest)) if first == JSON => (Form::Json, rest.to_vec()),
        _ => (Form::Rendered, args),
    }
}

/// **The refusal a trailing `--json` earns**, which is the arity refusal plus
/// the sentence that turns it from a puzzle into a typo. `lernie workspaces
/// --json` is what everybody types first — it is what the sibling tools take —
/// and being told that `workspaces` takes no argument answers a question the
/// operator did not ask.
fn misplaced(refusal: String, arguments: &[&str]) -> String {
    if arguments.contains(&JSON) {
        return format!("{refusal} — `{JSON}` goes BEFORE the word: `lernie {JSON} …`");
    }
    refusal
}

/// A hand-written envelope. **Read here**, in the pure function, because
/// whether a body is a gesture is decided entirely by what was typed — so it is
/// the caller's typo, it earns the usage, and it costs no connection.
fn ask(text: &str, form: Form) -> Decided {
    match crate::envelope::parse(text) {
        Ok(envelope) => Decided::Ask(envelope, form),
        Err(refusal) => Decided::Say(Verdict::refused(refusal)),
    }
}

/// **An enrollment, with the two arguments this binary can settle itself.**
///
/// The optional tail is read here for the same reason: `--into <dir>` is a
/// destination on THIS box, so whether it was spelled correctly is decided
/// entirely by what was typed. A tail that is anything else refuses naming the
/// one word, rather than falling through to the verb table and earning an arity
/// sentence about three arguments that says nothing about the fourth.
///
/// `grade` is a closed set of two words the boundary defines (REMOTE §8.4) and
/// this binary already holds them — `lernie help enroll` says so in its own
/// words. So a typo is read here, in the pure function, for exactly the reason
/// [`ask`] reads a body here: it is decided entirely by what was typed, it is
/// the caller's typo, it earns the usage, and it costs no connection (bl-07b9).
/// It used to cost a full round trip and come back `unknown grade "OPERATOR"` —
/// true, and naming neither of the two words that would have worked.
///
/// **It is not a second authority on grades.** The engine stays the place that
/// decides what a grade means and whether this box may ask for one at all —
/// §8.4 refuses the act unless this box's own leaf is operator-grade, which is
/// not knowable here. What is settled here is only whether the word is one of
/// the two, read off [`crate::ui::Grade`]'s own list rather than a second copy
/// of it.
fn enroll(workspace: &str, name: &str, grade: &str, tail: &[&str]) -> Decided {
    let into = match tail {
        [] => None,
        [word, dir] if *word == INTO => Some((*dir).to_owned()),
        _ => {
            return Decided::Say(Verdict::refused(format!(
                "`lernie enroll` takes one optional word after the three, and it is \
                 `{INTO} <dir>` — not {:?}",
                tail.join(" ")
            )));
        }
    };
    let words = crate::ui::Grade::both();
    let Some(held) = words.iter().find(|known| known.word() == grade) else {
        return Decided::Say(Verdict::refused(format!(
            "unknown grade {grade:?} — `lernie enroll` takes {}",
            words
                .iter()
                .map(|known| format!("{:?}", known.word()))
                .collect::<Vec<String>>()
                .join(" or ")
        )));
    };
    Decided::Enroll {
        workspace: workspace.to_owned(),
        name: name.to_owned(),
        grade: held.word(),
        into,
    }
}

/// The one word `enroll` takes after its three, and the one place it is spelled
/// — the pattern that reads it and the refusal that teaches it both name this.
pub const INTO: &str = "--into";

/// A typed verb, a structural door, or a first word that is neither.
///
/// The whole argument list still decides — a word with the wrong number of
/// arguments refuses rather than ignoring the extras — but the refusal it earns
/// names the word and its grammar, where a word that is no word at all can only
/// be quoted back.
///
/// **A door reaching here is always a wrong arity** (bl-6bda), because every
/// door's exact spelling is matched above. It used to fall through to the
/// quote-back, so `lernie entries x y` and `lernie help a b` were told they
/// were not arguments this binary recognises — the sentence a genuine typo
/// earns, about words the usage lists one screen up.
fn typed(word: &str, arguments: &[&str], form: Form) -> Decided {
    if let Some(verb) = crate::verbs::find(word) {
        return match verb.envelope(arguments.iter().map(|a| (*a).to_owned()).collect()) {
            Ok(envelope) if verb.addresses_a_workspace() => Decided::Ask(envelope, form),
            Ok(envelope) => Decided::Fanned(envelope, form),
            Err(refusal) => Decided::Say(Verdict::refused(misplaced(refusal, arguments))),
        };
    }
    if let Some(door) = crate::verbs::doors::find(word) {
        return Decided::Say(Verdict::refused(misplaced(
            door.refused(arguments.len()),
            arguments,
        )));
    }
    Decided::Say(Verdict::refused(format!(
        "unrecognised argument: {}",
        std::iter::once(word)
            .chain(arguments.iter().copied())
            .collect::<Vec<&str>>()
            .join(" ")
    )))
}

#[cfg(test)]
mod tests;
