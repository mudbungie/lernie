//! **What this binary says about itself**, and how a verdict carries it: the
//! version line, the usage, and the four constructors' invariants — the exit
//! code, the stream, and which of them wears the usage.

use super::super::verdict::{FAILED, REFUSED};
use super::super::{Stream, Verdict, usage, version};

#[test]
fn version_names_the_crate_and_its_version() {
    assert_eq!(version(), format!("lernie {}", env!("CARGO_PKG_VERSION")));
}

/// **The version is the fence**, so the number this build prints must be on the
/// seat's side of it: 0.0.x under this name was the engine, which continues as
/// litany.
#[test]
fn the_version_is_on_the_seat_s_side_of_the_fence() {
    let stated = env!("CARGO_PKG_VERSION");
    let (major, rest) = stated.split_once('.').expect("a dotted version");
    let minor: u32 = rest
        .split('.')
        .next()
        .expect("a minor")
        .parse()
        .expect("a number");
    let major: u32 = major.parse().expect("a number");
    assert!(
        major > 0 || minor > 0,
        "{stated} is a 0.0.x version, which under this name is the engine"
    );
}

#[test]
fn usage_leads_with_the_version_line_then_says_what_lernie_is() {
    let text = usage();
    assert!(
        text.starts_with(&version()),
        "usage did not lead with the version: {text}"
    );
    assert!(text.contains("lernie is the seat"), "{text}");
    assert!(text.contains("executes nothing"), "{text}");
}

/// The usage states the fence, because the name has two eras in the published
/// record and a caller who typed `lernie` may be holding the other one.
#[test]
fn usage_states_the_fence_and_names_the_engine_s_new_name() {
    let text = usage();
    assert!(text.contains("TWO ERAS"), "{text}");
    assert!(text.contains("litany"), "{text}");
    assert!(text.contains("0.1.0 and above is this seat"), "{text}");
}

/// The usage names the command line's own words, the two paths an operator has
/// to fill by hand, and **every typed verb** — the last derived from the table
/// rather than restated, so a verb added tomorrow is in the usage the moment it
/// is in the roster.
#[test]
fn usage_names_the_verbs_and_what_it_reads() {
    let text = usage();
    assert!(text.contains("lernie entries"), "{text}");
    assert!(text.contains("lernie start <workspace> <goal>"), "{text}");
    assert!(text.contains("lernie ask <envelope>"), "{text}");
    assert!(text.contains("lernie help [<verb>]"), "{text}");
    assert!(text.contains("wire/workspaces/"), "{text}");
    assert!(text.contains("$XDG_DATA_HOME/lernie"), "{text}");
    for verb in crate::verbs::table() {
        assert!(
            text.contains(&verb.usage()),
            "{} is not in the usage",
            verb.word
        );
    }
}

/// **The usage says that bare `lernie` opens the window** (bl-7dcf).
///
/// It is the crate's headline capability and it was in neither list: the shape
/// read `lernie <verb> [argument…]`, which says a verb is REQUIRED — the
/// opposite of the truth — and the only route to the window from the help was
/// running the binary wrong on purpose. The prose does not rescue it either:
/// *"paints what comes back"* reads as the verbs' own stdout, because every
/// documented verb prints envelopes there.
#[test]
fn the_usage_says_that_a_bare_invocation_opens_the_window() {
    let text = usage();
    assert!(
        text.lines()
            .any(|line| line.starts_with("usage: lernie ") && line.contains("window")),
        "the bare form leads the usage block:\n{text}"
    );
    assert!(
        text.contains("WITH NO ARGUMENTS AT ALL"),
        "and the prose says which invocation it is:\n{text}"
    );
    assert!(
        matches!(crate::cli::run(Vec::new()), crate::cli::Decided::Window),
        "and that is what a bare invocation actually decides"
    );
}

#[test]
fn ok_carries_the_success_code_and_goes_to_stdout() {
    let v = Verdict::ok("said".to_string());
    assert_eq!(v.code, 0);
    assert_eq!(v.text, "said");
    assert_eq!(v.stream, Stream::Out);
}

/// **An answer is a product whichever way it went.** The engine's `ok: false`
/// is still stdout; only the code says no.
#[test]
fn an_answer_goes_to_stdout_either_way_and_only_the_code_says_no() {
    let yes = Verdict::answered("frames".to_string(), true);
    assert_eq!((yes.code, yes.stream), (0, Stream::Out));
    let no = Verdict::answered("frames".to_string(), false);
    assert_eq!((no.code, no.stream), (FAILED, Stream::Out));
    assert_eq!(no.text, "frames", "an answer is never decorated");
}

/// The prefix and the usage are the constructor's, not the call site's — so
/// this holds for a refusal nobody has written yet.
#[test]
fn every_refusal_names_what_it_refused_and_still_teaches() {
    let v = Verdict::refused("that is not a verb".to_string());
    assert_eq!(v.code, REFUSED);
    assert_eq!(v.stream, Stream::Err);
    assert_eq!(v.text, format!("lernie: that is not a verb\n\n{}", usage()));
}

/// **A failure carries no usage**, and that is the difference: a refusal is
/// about what the caller typed, a failure is about this box or the far end,
/// where a usage line is noise in front of the sentence that matters.
#[test]
fn a_failure_says_only_what_happened() {
    let v = Verdict::failed("this box holds no channel".to_string());
    assert_eq!(v.code, FAILED);
    assert_eq!(v.stream, Stream::Err);
    assert_eq!(v.text, "lernie: this box holds no channel");
    assert!(!v.text.contains("usage:"), "{}", v.text);
}
