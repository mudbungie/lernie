//! **`enroll`'s grammar** — the one word on this surface whose tail is more
//! than its parameters, in its own file for the reason yog's own line reader
//! cut the same seam at (`src/boundary/line/enroll.rs`): *a verb whose grammar
//! is more than words is its own file*. It took [`super`] past the 300-line
//! wall when the second optional word landed (bl-971c).
//!
//! Everything here is read in the pure function, because everything here is
//! decided entirely by what was typed — the two optional words, and the grade,
//! which is a closed set of two this binary already holds.

use super::{Decided, Verdict};

/// **An enrollment, with the two arguments this binary can settle itself.**
///
/// The optional tail is read here for the same reason: `--into <dir>` is a
/// destination on THIS box, so whether it was spelled correctly is decided
/// entirely by what was typed. A tail that is anything else refuses naming the
/// words it takes, rather than falling through to the verb table and earning an
/// arity sentence about three arguments that says nothing about the fourth.
///
/// **Two words, both optional, independent and in either order** (bl-971c).
/// `--at <host>:<port>` is the route the enrolled DEVICE will dial (REMOTE §8.4
/// as amended, yog bl-fec6) and `--into <dir>` is where this box writes the
/// material down; neither implies the other, and a device behind an alias
/// needs the first whether or not it needs the second. What the seat settles
/// about `--at` is only that the word was given a value: whether the value
/// names an endpoint a device can dial is the ENGINE's judgement, which it
/// makes on its own address and a stated one alike and answers with the remedy
/// (`yog wire-certs`), so a copy of that rule here would be a second authority
/// on addresses that could disagree with the first.
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
pub(super) fn enroll(workspace: &str, name: &str, grade: &str, tail: &[&str]) -> Decided {
    let (at, into) = match stated(tail) {
        Ok(pair) => pair,
        Err(refusal) => return Decided::Say(Verdict::refused(refusal)),
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
        at,
        into,
    }
}

/// **`enroll`'s tail, read as the two words it is** — each at most once, in
/// either order, each with its value — or the refusal that teaches both.
///
/// One refusal for every way the tail can be wrong, because to the operator
/// they are one event: *this is not a tail `enroll` takes, and here is the tail
/// it takes*. Naming the grammar answers a repeat, an unknown word and a word
/// with no value at once, where three sentences would each answer a third of a
/// question nobody asked in thirds.
fn stated(tail: &[&str]) -> Result<(Option<String>, Option<String>), String> {
    let (mut at, mut into) = (None, None);
    let mut rest = tail;
    while let [word, value, more @ ..] = rest {
        let slot = match *word {
            AT => &mut at,
            INTO => &mut into,
            _ => return Err(tailed(tail)),
        };
        if slot.is_some() {
            return Err(tailed(tail));
        }
        *slot = Some((*value).to_owned());
        rest = more;
    }
    if rest.is_empty() {
        return Ok((at, into));
    }
    Err(tailed(tail))
}

/// The sentence that teaches the tail, quoting back what was typed.
fn tailed(tail: &[&str]) -> String {
    format!(
        "`lernie enroll` takes two optional words after the three — `{AT} <host>:<port>`, the \
         address the enrolled box will dial, and `{INTO} <dir>`, where to write the material \
         down — each at most once, with its value, in either order. Not {:?}",
        tail.join(" ")
    )
}

/// The two words `enroll` takes after its three, and the one place each is
/// spelled — the pattern that reads it and the refusal that teaches it both
/// name these.
pub const INTO: &str = "--into";

/// The route the enrolled device will dial, which becomes
/// [`crate::verbs::ADDRESS`] on the wire. It is `--at` and not `--address`
/// because it reads as the preposition it is, and because the engine's own
/// line spells it that way (yog's `/enroll <name> [foot] --at <host>:<port>`),
/// so an operator who learned it at one face has learned it at both.
pub const AT: &str = "--at";
