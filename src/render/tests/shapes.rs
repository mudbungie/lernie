//! **The arms the corpus cannot reach.** It carries one fixture a kind, so a
//! rendering with four classes inside it — a preview, a captured document, a
//! transcript entry, a block — is exercised on exactly one of them. These are
//! the rest, built as frames rather than as typed values so they go through the
//! real reader on the way in.

use serde_json::{Value, json};

use crate::render::rendered;

/// Render a frame and assert every needle is in it.
#[track_caller]
fn says(frame: &Value, needles: &[&str]) {
    let said = rendered(frame);
    for needle in needles {
        assert!(said.contains(needle), "{frame}\n-> {said}");
    }
}

/// **The four classes a bounded preview can be**, on the two surfaces that
/// share one reader: a worktree's file preview and a step's capture logs.
#[test]
fn every_class_of_a_bounded_preview_says_what_it_is() {
    for (preview, needle) in [
        (json!({"kind": "text", "text": "hello"}), "hello"),
        (
            json!({"kind": "truncated", "text": "hel", "size": 9_000}),
            "head of 9000 bytes",
        ),
        (json!({"kind": "binary", "size": 12}), "12 bytes, binary"),
        (json!({"kind": "a-class-from-2027"}), "no reading"),
    ] {
        says(
            &json!({"ok": true, "kind": "files", "worktree": true, "rows": [],
                    "truncated": false, "preview": preview}),
            &[needle],
        );
    }
}

/// A worktree that answered no listing at all is a different claim from a
/// worktree that answered an empty one, and both are painted.
#[test]
fn a_worktree_that_answered_nothing_says_so() {
    says(
        &json!({"ok": true, "kind": "files", "worktree": false}),
        &["no worktree listing"],
    );
    says(
        &json!({"ok": true, "kind": "files", "worktree": true, "truncated": true,
                "working_dir": "/w", "rows": [
                    {"path": "src", "size": 0, "dir": true},
                    {"path": "src/lib.rs", "size": 40, "dir": false}]}),
        &["in /w", "(truncated)", "src/", "40 bytes"],
    );
}

/// **The four classes a captured document can be**, printed as what they are
/// and never parsed.
#[test]
fn every_class_of_a_captured_document_is_printed_as_what_it_is() {
    let absent = json!({"kind": "absent"});
    says(
        &json!({"ok": true, "kind": "step", "seq": "004",
                "meta": {"kind": "json", "raw": "{\"a\":1}"},
                "request": absent,
                "staging": {"kind": "unparsed", "note": "not JSON", "raw": "oops"},
                "response": [{"kind": "a-class-from-2027"}],
                "tools": [{"tool_id": "t1", "is_error": true,
                           "input": absent, "output": absent}],
                "stderr": {"kind": "text", "text": "warned"},
                "driver": {"kind": "text", "text": "drove"}}),
        &[
            "step 004",
            "{\"a\":1}",
            "request: not written",
            "not JSON",
            "no reading",
            "t1",
            "ERROR",
            "warned",
            "drove",
        ],
    );
}

/// **Every kind of transcript entry, and every kind of block in a model's
/// turn.** The corpus carries one conversation; these are the rest of the
/// vocabulary.
#[test]
fn every_transcript_entry_and_every_block_has_a_reading() {
    says(
        &json!({"ok": true, "kind": "transcript", "rows": [
            {"name": "002-model.md", "raw": "", "kind": "model",
             "model_id": "a-model", "usage": {"input": 5},
             "blocks": [{"kind": "text", "text": "said"},
                        {"kind": "thinking", "text": "thought"},
                        {"kind": "tool-use", "id": "t1", "name": "Bash", "input": "ls"},
                        {"kind": "a-block-from-2027"}]},
            {"name": "003-tool.md", "raw": "", "kind": "tool-result",
             "tool_use_id": "t1", "content": "ok", "is_error": true},
            {"name": "004-live.md", "raw": "", "kind": "streaming",
             "thinking": "still", "text": "going"},
            {"name": "005-fold.md", "raw": "", "kind": "compacted",
             "first": 1, "last": 9, "summary": "the gist"},
            {"name": "006-raw.md", "raw": "", "kind": "raw"},
            {"name": "007-new.md", "raw": "", "kind": "an-entry-from-2027"}
        ]}),
        &[
            "a-model",
            "input 5",
            "said",
            "thinking",
            "thought",
            "Bash (t1)",
            "[a-block-from-2027]",
            "result of t1",
            "ERROR",
            "still",
            "going",
            "compacted 1–9",
            "the gist",
            "raw",
            "an-entry-from-2027",
        ],
    );
}

/// **The receipts that carry a boolean**, both ways round. Each says what the
/// engine said rather than what was asked — a restore of a conversation whose
/// ancestor is floored leaves it floored, and the receipt is what says so.
#[test]
fn every_boolean_receipt_says_both_of_its_answers() {
    for (frame, needle) in [
        (
            json!({"ok": true, "kind": "retired", "discarded": true}),
            "went with it",
        ),
        (
            json!({"ok": true, "kind": "retired", "discarded": false}),
            "is kept",
        ),
        (
            json!({"ok": true, "kind": "floored", "standing": true}),
            "a floor stands",
        ),
        (
            json!({"ok": true, "kind": "floored", "standing": false}),
            "no floor stands",
        ),
        (
            json!({"ok": true, "kind": "armed", "armed": true}),
            "it is armed",
        ),
        (
            json!({"ok": true, "kind": "armed", "armed": false}),
            "not armed",
        ),
        (
            json!({"ok": true, "kind": "answered", "tool": "Bash", "tool_use": "t1",
                   "verdict": "allow", "advanced": true}),
            "advanced",
        ),
        (
            json!({"ok": true, "kind": "answered", "tool": "Bash", "tool_use": "t1",
                   "verdict": "hold", "advanced": false}),
            "still parked",
        ),
        (
            json!({"ok": true, "kind": "delivered", "base": "b", "target": "main",
                   "source": "work/x", "commit": "abc"}),
            "as abc",
        ),
        (
            json!({"ok": true, "kind": "delivered", "base": "b", "target": "main",
                   "source": null, "commit": null}),
            "landed nothing",
        ),
    ] {
        says(&frame, &[needle]);
    }
}

/// **The two wordings of how a policy is settled**, and neither is composed
/// here: `reply::governing` holds the engine's own sentence.
#[test]
fn a_governing_commit_carries_the_engines_own_sentence_either_way() {
    says(
        &json!({"ok": true, "kind": "governing", "oid": "abc123", "short_oid": "abc123",
                "follows": "main", "files": ["souls/worker.md"]}),
        &["policy follows config/main", "souls/worker.md"],
    );
    says(
        &json!({"ok": true, "kind": "governing", "oid": "abc123", "short_oid": "abc123",
                "follows": null, "diverged_lineages": 2, "files": []}),
        &["policy held at abc123", "no file governs it"],
    );
}

/// **A staged start, a spread of them, and the enrollment whose material is
/// deliberately not printed.**
#[test]
fn the_start_family_renders_and_an_enrollment_prints_no_secret() {
    let body = json!({"workspace": "proj", "goal": "write the changelog"});
    says(
        &json!({"ok": true, "kind": "prepared", "prepared": body}),
        &["staged in proj", "write the changelog"],
    );
    says(
        &json!({"ok": true, "kind": "fanned", "rows": []}),
        &["no candidate"],
    );
    let said = rendered(&json!({"ok": true, "kind": "enrolled", "grade": "operator",
                                "name": "phone", "address": "h:1", "ca": "CA-PEM",
                                "cert": "CERT-PEM", "key": "KEY-PEM"}));
    assert!(said.contains("enrolled phone"), "{said}");
    assert!(
        !said.contains("KEY-PEM"),
        "the key reached the terminal: {said}"
    );
}

/// **All three gates, because the corpus's own conversation offers two.** The
/// offers are where the seat says a cascade is on the table (bl-9fd1), so the
/// word for each has to be reachable — and the fixture upstream ships is a
/// conversation nothing may be nudged onto.
#[test]
fn every_gate_the_engine_can_offer_has_a_word() {
    let path = crate::test_support::corpus::root()
        .join("answers")
        .join("agent.json");
    let mut frame = crate::test_support::corpus::fixture(&path)
        .frames
        .first()
        .cloned()
        .expect("the agent fixture carries a frame");
    for gate in ["nudgeable", "stoppable", "stop_children"] {
        frame[gate] = Value::Bool(true);
    }
    says(&frame, &["offers: nudge, stop, stop children"]);
}

/// **The trail names the author** (REMOTE §9.20): the identity that made the
/// act, beside the origin rather than in a detail, because it is what tells two
/// rows spelling the same `argv`, `cwd` and `exit` apart.
#[test]
fn a_trail_row_says_who_made_the_act() {
    says(
        &json!({"ok": true, "kind": "ops", "rows": [
            {"ts": "1700", "client": "phone-1", "origin": "balls", "standing": "clean",
             "failed": false, "exit_label": "exit 0", "exit": 0, "argv": "bl list",
             "cwd": "/p", "stdout": "", "stderr": ""}]}),
        &["phone-1", "balls", "bl list"],
    );
}

/// **A machine that has never dialled says so** (REMOTE §5, PROTOCOL 14), and
/// it is the row an operator can safely remove. On this surface presence reads
/// false for every row — every verb opens and closes its own connection — so
/// the stamp's absence is the only thing here that chooses.
#[test]
fn a_client_that_has_never_dialled_is_the_row_that_says_it() {
    // Absent rather than null, which is the wire's own spelling of it
    // (REMOTE §5: "absent for a client that has never dialled").
    says(
        &json!({"ok": true, "kind": "clients", "rows": [
            {"client": "minted", "present": false, "tools": []}]}),
        &["minted", "never dialled"],
    );
    let dialled = rendered(&json!({"ok": true, "kind": "clients", "rows": [
        {"client": "real", "present": false, "tools": [], "last_seen": 1_757_000_000_i64}]}));
    assert!(!dialled.contains("never dialled"), "{dialled}");
}
