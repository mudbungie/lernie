//! Every decision one invocation can make, read back as a value.

use super::{Decided, FAILED, REFUSED, Stream, Verdict, run, usage, version};

/// Build the argument vector the way `main` does, from string literals.
fn argv(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| (*w).to_string()).collect()
}

/// What a run said, for the arguments that decide to say something.
fn said(words: &[&str]) -> Verdict {
    match run(argv(words)) {
        Decided::Say(verdict) => verdict,
        Decided::Entries => panic!("{words:?} decided to list entries"),
        Decided::Ask(_) => panic!("{words:?} decided to ask"),
    }
}

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

/// The usage names both verbs and the two paths an operator has to fill by
/// hand, because a seat can do nothing until they exist.
#[test]
fn usage_names_the_verbs_and_what_it_reads() {
    let text = usage();
    assert!(text.contains("lernie entries"), "{text}");
    assert!(text.contains("lernie ask <envelope>"), "{text}");
    assert!(text.contains("wire/workspaces/"), "{text}");
    assert!(text.contains("$XDG_DATA_HOME/lernie"), "{text}");
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

#[test]
fn both_version_spellings_print_the_version_and_succeed() {
    for spelling in ["--version", "-V"] {
        let v = said(&[spelling]);
        assert_eq!(v.code, 0, "{spelling} did not succeed");
        assert_eq!(v.text, version(), "{spelling} printed something else");
    }
}

#[test]
fn both_help_spellings_print_the_usage_and_succeed() {
    for spelling in ["--help", "-h"] {
        let v = said(&[spelling]);
        assert_eq!(v.code, 0, "{spelling} did not succeed");
        assert_eq!(v.text, usage(), "{spelling} printed something else");
    }
}

/// The two verbs decide, and say nothing: what they do needs this process's own
/// environment, which is the entry point's to fold.
#[test]
fn the_verbs_decide_and_carry_what_they_were_given() {
    assert!(matches!(run(argv(&["entries"])), Decided::Entries));
    match run(argv(&["ask", r#"{"op":"workspaces"}"#])) {
        Decided::Ask(envelope) => assert_eq!(envelope, r#"{"op":"workspaces"}"#),
        _ => panic!("ask did not decide to ask"),
    }
}

/// **A bare invocation names the state of the tree**: the window is what a seat
/// is for, and it is not built, so the refusal says so instead of implying the
/// caller mistyped something.
#[test]
fn a_bare_invocation_refuses_and_says_the_window_is_not_built() {
    let v = said(&[]);
    assert_eq!(v.code, REFUSED);
    assert!(v.text.contains("the window is not built yet"), "{}", v.text);
    assert!(v.text.contains("usage: lernie"), "{}", v.text);
}

#[test]
fn ask_with_no_envelope_says_what_it_wanted() {
    let v = said(&["ask"]);
    assert_eq!(v.code, REFUSED);
    assert!(v.text.contains("wants one gesture envelope"), "{}", v.text);
}

#[test]
fn an_unrecognised_argument_refuses_and_quotes_every_word_of_it() {
    let v = said(&["seat", "--ws", "Example"]);
    assert_eq!(v.code, REFUSED);
    assert!(
        v.text.contains("unrecognised argument: seat --ws Example"),
        "{}",
        v.text
    );
}

/// The match is on the WHOLE argument list, not on a first word, so a word that
/// would succeed alone refuses when something rides behind it — rather than
/// silently ignoring the rest.
#[test]
fn a_recognised_word_with_extra_words_is_not_recognised() {
    for extra in [["--version", "--now"], ["entries", "--now"]] {
        let v = said(&extra);
        assert_eq!(v.code, REFUSED);
        assert!(
            v.text
                .contains(&format!("unrecognised argument: {}", extra.join(" "))),
            "{}",
            v.text
        );
    }
    let v = said(&["ask", "{}", "twice"]);
    assert_eq!(v.code, REFUSED);
}
