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

/// What one invocation decided to do.
mod decided;

/// `answer`'s own grammar: three words and the optional reach of the fourth.
mod answer;
/// `enroll`'s own grammar: three words and two optional ones.
mod enroll;
/// `start`'s own grammar: the rung's positional word, and the role's written one.
mod start;

pub use decided::{Asking, Decided};
pub use enroll::{AT, INTO};
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
        ["start", address, goal, tail @ ..] => start::start(address, goal, tail, form),
        // Ahead of the typed table, because the word spends its row MORE THAN
        // ONCE (bl-f076): a read ends at the engine's step boundary and the
        // question an operator asked is about the whole of the work. `follow`
        // stays a row of the gesture table — the window's own lane spends it,
        // and the corpus round-trips it — so a wrong arity still earns the
        // row's own usage one arm down.
        ["follow", workspace, agent] => Decided::Follow {
            workspace: (*workspace).to_owned(),
            agent: (*agent).to_owned(),
            form,
        },
        // Ahead of the typed table for [`Follow`](Decided::Follow)'s reason
        // one noun over: the word spends a SECOND row first (bl-1e5a). The
        // assignment is unchanged and is still the table's; what is added
        // ahead of it is the read that says whether the id is one the provider
        // offers.
        ["model", workspace, role, provider, model] => Decided::Model {
            workspace: (*workspace).to_owned(),
            role: (*role).to_owned(),
            provider: (*provider).to_owned(),
            model: (*model).to_owned(),
            form,
        },
        // **The trail, with the depth defaulted** (bl-28a4). `ops` carries a
        // NUMBER, so it is a door rather than a row (`crate::verbs::doors`),
        // and the wire refuses an envelope without one — so the word answers
        // that question from the constant the window's own pane asks with
        // rather than making an operator answer it.
        ["ops"] => Decided::Fanned(crate::verbs::ops(crate::verbs::DEPTH), form),
        ["ops", depth] => trail(depth, form),
        // **The capability answer, with its reach defaulted** (PROTOCOL 18;
        // `ops`' own shape one noun over). The wire requires `scope` in both
        // directions and an operator may leave it off, which no row of named
        // strings can express — so the word answers the field from the narrow
        // constant when nothing was typed, and the wider two are the fourth
        // word. The row stays in the table: the window spends it, the corpus
        // round-trips it, and a wrong arity still earns its usage below.
        ["answer", workspace, agent, verdict] => {
            answer::answer(workspace, agent, verdict, None, form)
        }
        ["answer", workspace, agent, verdict, scope] => {
            answer::answer(workspace, agent, verdict, Some(scope), form)
        }
        // Ahead of the typed table, and only because of what the answer
        // carries: the row is the same row, and the envelope is built from it.
        ["enroll", workspace, name, grade, tail @ ..] => {
            enroll::enroll(workspace, name, grade, tail, form)
        }
        // The bare invocation is the window, because a seat is a window. Every
        // other spelling is a way of reaching one gesture without one.
        [] => Decided::Window,
        [word, arguments @ ..] => typed(word, arguments, form),
    }
}

/// **The trail at a stated depth**, or the refusal a word that is not a number
/// earns.
///
/// Read here, in the pure function, for exactly the reason [`ask`]'s body and
/// [`enroll`]'s grade are: whether `max` is a number is decided entirely by
/// what was typed, so it is the caller's typo, it earns the usage, and it
/// costs no connection. The wire would refuse it too, in its own words
/// (`non-integer field "max"`), a round trip later.
fn trail(depth: &str, form: Form) -> Decided {
    let Ok(max) = depth.parse::<u64>() else {
        return Decided::Say(Verdict::refused(misplaced(
            format!(
                "`lernie ops` takes a depth in rows and got {depth:?} — usage: {}",
                crate::verbs::doors::OPS.usage()
            ),
            &[depth],
        )));
    };
    Decided::Fanned(crate::verbs::ops(max), form)
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
