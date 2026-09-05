//! **The records pane's own fixture** — the model with it open and answered
//! (bl-2cf7, bl-b52c).
//!
//! Split from [`super`] at the design-time budget on the seam the pane draws:
//! it is the one covering pane with four reads under it, and the two balls
//! that grew it are not the last. Everything else about a focused pane stays
//! there; what a records answer looks like lives here.

use super::super::seated;
use crate::ui::Model;

/// One step, complete and quiet — the row with nothing to complain about.
pub(crate) fn step(seq: &str) -> crate::reply::steps::StepRow {
    crate::reply::steps::StepRow {
        seq: seq.to_owned(),
        framing: "complete".to_owned(),
        attempts: 1,
        tokens: crate::reply::steps::Spend {
            input: 11,
            output: 22,
            cache_read: 33,
            cache_write: 44,
            total: 99,
        },
        commit: Some("abcdef1".to_owned()),
        started_at: Some("2026-08-30T05:12Z".to_owned()),
        ended_at: Some("2026-08-30T05:14Z".to_owned()),
        auth_row: None,
        wound: crate::reply::steps::NONE.to_owned(),
        wound_reason: None,
    }
}

/// One notch, operable and seated in the chat — the row a fork control hangs
/// on.
pub(crate) fn notch(seq: &str) -> crate::reply::rail::Notch {
    crate::reply::rail::Notch {
        seq: seq.to_owned(),
        commit: Some("abcdef1234567890".to_owned()),
        budget: 120,
        seat: Some(crate::reply::rail::Seat {
            row: "003-claude.json".to_owned(),
            cut: 2,
        }),
    }
}

/// **The seated model with the records pane open and answered** (bl-2cf7,
/// bl-b52c): a quiet step and a wounded one, a walked worktree with work
/// landing elsewhere, a spine with one operable notch and one unreachable
/// one, a child hanging off it, and the config commit governing the whole —
/// every sentence the pane can say, on one screen.
pub(crate) fn recorded() -> Model {
    let wounded = crate::reply::steps::StepRow {
        seq: "002".to_owned(),
        framing: "failed".to_owned(),
        attempts: 2,
        commit: None,
        started_at: None,
        ended_at: None,
        auth_row: Some("housevendor".to_owned()),
        wound: crate::reply::steps::REFUSED.to_owned(),
        wound_reason: Some("no bytes".to_owned()),
        ..step("002")
    };
    Model {
        listing: Some(crate::ui::Listing::Records),
        steps: Some(crate::reply::steps::Steps {
            rows: vec![step("001"), wounded],
            orphan: "mail".to_owned(),
            orphan_reason: Some("driver died".to_owned()),
        }),
        files: Some(crate::reply::files::Files {
            listing: Some(crate::reply::files::Listing {
                rows: vec![
                    crate::reply::files::FileRow {
                        path: "src".to_owned(),
                        size: 0,
                        dir: true,
                    },
                    crate::reply::files::FileRow {
                        path: "src/a.rs".to_owned(),
                        size: 12,
                        dir: false,
                    },
                ],
                truncated: true,
            }),
            preview: None,
            working_dir: Some("/home/u/elsewhere".to_owned()),
        }),
        rail: Some(crate::reply::rail::Rail {
            notches: vec![
                notch("001"),
                crate::reply::rail::Notch {
                    seq: "002".to_owned(),
                    commit: None,
                    seat: None,
                    ..notch("002")
                },
            ],
            cards: vec![crate::reply::rail::Card {
                agent: "20260830T051200Z-a1b2-c".to_owned(),
                name: "Cobalt".to_owned(),
                fork: "from here".to_owned(),
                state: crate::reply::convs::AgentState::Live,
                tokens: 9,
                tail: Some("working".to_owned()),
                notch: 0,
            }],
        }),
        governing: Some(crate::reply::governing::Governing {
            oid: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_owned(),
            short_oid: "bbbbbbbb".to_owned(),
            governance: crate::reply::governing::Governance::Follows("default".to_owned()),
            files: vec!["workflow.yaml".to_owned()],
        }),
        ..seated()
    }
}
