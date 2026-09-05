//! The attempt's reading: the whole row, the absences, the nested counters and
//! outcome, and rung 1's refusals.

use serde_json::json;

use super::row;

/// A whole attempt, read field for field.
#[test]
fn an_attempt_carries_its_inputs_its_figures_its_verdicts_and_its_ending() {
    let read = row(&json!({
        "base": "f00dbeef", "compacted": 12,
        "conversation": "20260815T101112Z-abcd1234",
        "diff": { "ball_id": "bl-1", "project": "p", "state": "unreadable" },
        "goal": "ship it", "governing": "deadbeef",
        "outcome": { "commit": "ccc", "state": "accepted" },
        "pins": ["instructions/00-AGENTS.md=/p/AGENTS.md"],
        "response": "done, tests green", "steps": 4,
        "usage": { "cache_read_tokens": 33, "cache_write_tokens": 44,
                   "input_tokens": 11, "output_tokens": 22 },
        "verdicts": [{ "body": "candidate B reads cleaner", "sender": "judge-one" }],
        "wall_secs": 90
    }))
    .expect("a whole attempt reads");
    assert_eq!(read.diff.ball_id, "bl-1");
    assert_eq!(read.base.as_deref(), Some("f00dbeef"));
    assert_eq!(
        read.conversation.as_deref(),
        Some("20260815T101112Z-abcd1234")
    );
    assert_eq!(read.goal.as_deref(), Some("ship it"));
    assert_eq!(read.governing.as_deref(), Some("deadbeef"));
    assert_eq!(read.response.as_deref(), Some("done, tests green"));
    assert_eq!(read.pins.len(), 1);
    assert_eq!(read.usage.input, 11);
    assert_eq!(read.usage.output, 22);
    assert_eq!(read.usage.cache_read, 33);
    assert_eq!(read.usage.cache_write, 44);
    assert_eq!(read.wall_secs, 90);
    assert_eq!(read.steps, 4);
    assert_eq!(
        read.verdicts.first().expect("a verdict").sender,
        "judge-one"
    );
    assert_eq!(
        read.verdicts.first().expect("a verdict").body,
        "candidate B reads cleaner"
    );
    assert_eq!(read.compacted, Some(12));
    assert_eq!(read.outcome.state, "accepted");
    assert_eq!(read.outcome.commit.as_deref(), Some("ccc"));
    assert!(read.outcome.by.is_none());
}

/// **Every absence is a reading.** An attempt with no conversation, no frozen
/// goal and no readable config has absences, not empty strings — and an intact
/// record carries no compaction count at all.
#[test]
fn a_bare_attempt_names_nothing_it_has_no_record_of() {
    let read = row(&json!({
        "diff": { "ball_id": "bl-2", "project": "p", "state": "unreadable" },
        "outcome": { "by": "at-0badcafe", "state": "rejected" },
        "pins": [], "steps": 4,
        "usage": { "cache_read_tokens": 0, "cache_write_tokens": 0,
                   "input_tokens": 0, "output_tokens": 0 },
        "verdicts": [], "wall_secs": 90
    }))
    .expect("a bare attempt reads");
    assert!(read.base.is_none());
    assert!(read.conversation.is_none());
    assert!(read.goal.is_none());
    assert!(read.governing.is_none());
    assert!(read.response.is_none());
    assert!(read.compacted.is_none());
    assert!(read.pins.is_empty());
    assert!(read.verdicts.is_empty());
    assert_eq!(read.outcome.by.as_deref(), Some("at-0badcafe"));
    assert!(read.outcome.commit.is_none());
}

/// **The outcome token rides verbatim**, so one this build has never seen
/// paints as itself rather than refusing the listing.
#[test]
fn an_outcome_this_build_has_never_seen_reads_as_itself() {
    let read = row(&json!({
        "diff": { "ball_id": "bl-2", "project": "p", "state": "unreadable" },
        "outcome": { "state": "quarantined" }, "pins": [], "steps": 0,
        "usage": { "cache_read_tokens": 0, "cache_write_tokens": 0,
                   "input_tokens": 0, "output_tokens": 0 },
        "verdicts": [], "wall_secs": 0
    }))
    .expect("an unknown outcome reads");
    assert_eq!(read.outcome.state, "quarantined");
}

/// Rung 1, at each of the four depths this answer has one.
#[test]
fn a_malformed_attempt_refuses_naming_what_was_wrong() {
    let bare = json!({
        "diff": { "ball_id": "b", "project": "p", "state": "unreadable" },
        "outcome": { "state": "pending" }, "pins": [], "steps": 0,
        "usage": { "cache_read_tokens": 0, "cache_write_tokens": 0,
                   "input_tokens": 0, "output_tokens": 0 },
        "verdicts": [], "wall_secs": 0
    });
    assert_eq!(
        row(&json!("row")),
        Err("science row: not an object".to_owned())
    );
    let mut without = bare.clone();
    without.as_object_mut().expect("an object").remove("diff");
    assert_eq!(row(&without), Err("science row: missing diff".to_owned()));
    let mut no_usage = bare.clone();
    no_usage.as_object_mut().expect("an object").remove("usage");
    assert_eq!(
        row(&no_usage),
        Err("missing or non-object field \"usage\"".to_owned())
    );
    let mut no_outcome = bare.clone();
    no_outcome
        .as_object_mut()
        .expect("an object")
        .remove("outcome");
    assert_eq!(
        row(&no_outcome),
        Err("missing or non-object field \"outcome\"".to_owned())
    );
    let mut bad_verdict = bare.clone();
    bad_verdict["verdicts"] = json!(["said"]);
    assert_eq!(row(&bad_verdict), Err("verdict: not an object".to_owned()));
    let mut half_verdict = bare;
    half_verdict["verdicts"] = json!([{ "sender": "one" }]);
    assert_eq!(
        row(&half_verdict),
        Err("missing or non-string field \"body\"".to_owned())
    );
}
