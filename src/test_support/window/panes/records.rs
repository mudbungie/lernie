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

/// The conversation's own row, live and busy — every optional fact stated, so
/// the header's every sentence is on one screen.
pub(crate) fn own_row() -> crate::reply::agent::Agent {
    crate::reply::agent::Agent {
        agent: "20260830T051200Z-a1b2".to_owned(),
        root: "20260830T051200Z-a1b2".to_owned(),
        ancestors: vec!["20260830T050000Z-root".to_owned()],
        display: "port the paint probe".to_owned(),
        display_only: true,
        tip: "aaaaaaa".to_owned(),
        state: crate::reply::convs::AgentState::InFlight,
        refused: true,
        failure: Some("no credential for provider row \"work\"".to_owned()),
        marks: vec!["notified".to_owned(), "held".to_owned()],
        flight: Some("tools".to_owned()),
        held: Some(crate::reply::queue::Held {
            tool: "Bash".to_owned(),
            tool_use: "toolu_1".to_owned(),
            reason: "unconfined".to_owned(),
        }),
        present: true,
        offers: vec![
            crate::reply::agent::Offer::Stop,
            crate::reply::agent::Offer::Children,
        ],
        seats: vec![crate::reply::agent::Seat {
            name: "pennant".to_owned(),
            doing: "waiting".to_owned(),
        }],
        strip: Some(crate::reply::agent::Strip {
            class: "tools".to_owned(),
            facts: "Bash · 5s".to_owned(),
        }),
        spend: crate::reply::spend::Figure {
            tokens: crate::reply::steps::Spend {
                input: 120,
                output: 0,
                cache_read: 0,
                cache_write: 0,
                total: 120,
            },
            usd: Some("$4.00".to_owned()),
            attribution: crate::reply::spend::Attribution {
                kind: "conversations".to_owned(),
                label: Some("over 3 conversations".to_owned()),
            },
        },
        context: Some(crate::reply::agent::Fullness {
            model: "claude-x".to_owned(),
            prompt_tokens: 4000,
            window: 200_000,
            percent: 2,
        }),
    }
}

/// One deposit, whole — a subagent's result, with the two facts only a result
/// message states.
pub(crate) fn deposit() -> crate::reply::inbox::Row {
    crate::reply::inbox::Row {
        name: "user-001.md".to_owned(),
        raw: "---\nfrom: user\n---\nhi".to_owned(),
        deposit: crate::reply::inbox::Deposit {
            from: Some("user".to_owned()),
            deposited_at: Some("2026-08-30T05:10Z".to_owned()),
            epitaph: Some("final-response".to_owned()),
            terminal_ref: Some("refs/litany/agents/c-1".to_owned()),
            body: "look at the elision".to_owned(),
        },
    }
}

/// One step's drill-in, with a record of every class the wire can carry.
pub(crate) fn drilled(seq: &str) -> crate::reply::step::Step {
    crate::reply::step::Step {
        seq: seq.to_owned(),
        meta: crate::reply::step::Doc::Json {
            raw: "{\"commit\":\"abcdef1\"}".to_owned(),
        },
        request: crate::reply::step::Doc::Absent,
        staging: crate::reply::step::Doc::Unparsed {
            note: crate::reply::step::UNPARSED.to_owned(),
            raw: "not json".to_owned(),
        },
        response: vec![crate::reply::step::Doc::Unknown("sideways".to_owned())],
        tools: vec![crate::reply::step::ToolCall {
            tool_id: "toolu_1".to_owned(),
            is_error: true,
            input: crate::reply::step::Doc::Absent,
            output: crate::reply::step::Doc::Unparsed {
                note: crate::reply::step::UNPARSED.to_owned(),
                raw: "raw".to_owned(),
            },
        }],
        stderr: Some(crate::reply::files::Preview::Truncated {
            text: "the adapter's last words".to_owned(),
            size: 999_999,
        }),
        driver: None,
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
        records: crate::ui::Records {
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
            agent: Some(own_row()),
            mail: Some(vec![deposit()]),
            governing: Some(crate::reply::governing::Governing {
                oid: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_owned(),
                short_oid: "bbbbbbbb".to_owned(),
                governance: crate::reply::governing::Governance::Follows("default".to_owned()),
                files: vec!["workflow.yaml".to_owned()],
            }),
            ..crate::ui::Records::default()
        },
        ..seated()
    }
}
