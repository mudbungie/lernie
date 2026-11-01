//! The act renderings the corpus cannot reach: a captured run on each of the
//! three shapes it takes.

use serde_json::json;

use crate::render::rendered;

/// **A run's receipt is the child's words, under a verdict only where the
/// words do not carry it** (bl-401b): a success that spoke is told in its
/// words alone — the engine's sentence verbatim, `ok` restated nothing and
/// read as this seat's prefix on it — a silent success says `ok` so it reads
/// apart from an act that never fired (bl-9760), and a failure says so with
/// the exit code over the child's own words.
#[test]
fn a_run_s_receipt_is_the_child_s_words_and_a_verdict_only_over_silence_or_failure() {
    let run = |exit: i32, stdout: &str, stderr: &str| {
        rendered(&json!({"ok": exit == 0, "kind": "outcome", "exit": exit,
                         "stdout": stdout, "stderr": stderr}))
    };
    assert_eq!(run(0, "", ""), "ok");
    assert_eq!(run(0, "", "  \n"), "ok");
    assert_eq!(
        run(
            0,
            "",
            "litany: config/default already governs [a1b2] — nothing to retarget\n"
        ),
        "litany: config/default already governs [a1b2] — nothing to retarget"
    );
    assert_eq!(run(0, "deposited\n", "warned"), "deposited\nwarned");
    assert_eq!(
        run(1, "", "unknown conversation"),
        "failed  exit 1\n  unknown conversation"
    );
}
