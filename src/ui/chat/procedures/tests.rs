use super::{line, markers};
use crate::reply::convs::AgentState;
use crate::reply::rail::{Card, Notch, Rail, Seat};

fn card(procedure: Option<&str>, landed: Option<&str>, tail: Option<&str>, notch: u64) -> Card {
    Card {
        agent: "c-1-t".to_owned(),
        name: "Tern".to_owned(),
        fork: "from here".to_owned(),
        state: AgentState::Quiescent,
        tokens: 12,
        tail: tail.map(str::to_owned),
        notch,
        procedure: procedure.map(str::to_owned),
        role: procedure.map(|_| "titler".to_owned()),
        landed: landed.map(str::to_owned),
    }
}

fn rail(cards: Vec<Card>) -> Rail {
    Rail {
        notches: vec![
            Notch {
                seq: "001".to_owned(),
                commit: Some("abcdef1234567890".to_owned()),
                budget: 0,
                cost: None,
                seat: Some(Seat {
                    row: "003-claude.json".to_owned(),
                    cut: 2,
                }),
            },
            Notch {
                seq: "002".to_owned(),
                commit: None,
                budget: 0,
                cost: None,
                seat: None,
            },
        ],
        cards,
    }
}

#[test]
fn a_procedure_card_stands_above_its_notchs_seat_and_nowhere_else() {
    let spine = rail(vec![
        card(
            Some("user_message"),
            Some("title"),
            Some("Port the probe"),
            0,
        ),
        // A subagent a model dispatched: a card, never a marker.
        card(None, None, Some("working"), 0),
        // A run whose notch the chat never seated paints nowhere.
        card(Some("worker_flush"), None, None, 1),
    ]);
    assert_eq!(
        markers(Some(&spine), "003-claude.json"),
        vec![
            "titler \u{b7} user_message \u{b7} landed title: \u{201c}Port the probe\u{201d} \u{b7} 12 tokens"
        ]
    );
    assert!(markers(Some(&spine), "001-user.md").is_empty());
    assert!(markers(None, "003-claude.json").is_empty());
}

#[test]
fn a_run_says_what_came_of_it_at_every_stage() {
    let mut running = card(Some("user_message"), None, None, 0);
    running.state = AgentState::InFlight;
    assert_eq!(
        line(&running),
        format!(
            "titler \u{b7} user_message \u{b7} {} \u{b7} 12 tokens",
            AgentState::InFlight.label()
        )
    );
    let silent = card(Some("worker_flush"), Some("summary"), None, 0);
    assert!(
        line(&silent).contains("landed summary \u{b7}"),
        "{}",
        line(&silent)
    );
    // A card with no role falls back to the server's display name.
    let mut nameless = card(Some("on_tool_return"), None, Some("CLEAN"), 0);
    nameless.role = None;
    assert!(
        line(&nameless).starts_with("Tern \u{b7} on_tool_return"),
        "{}",
        line(&nameless)
    );
}
