//! The two panels: the conversation's facts over its transcript, and the
//! seat's where nothing is selected.

use super::{WAYS, version, ways_in};
use crate::test_support::window::{painted, recorded, seated};
use crate::ui::Model;

/// **The conversation's card** names it, where it is, what answered and
/// what it cost, and where it hangs.
#[test]
fn the_conversation_s_card_carries_its_facts() {
    let mut model = recorded();
    model.listing = None;
    let text = painted(&mut model);
    for word in [
        "port the paint probe",
        "on home — (this box's own server)",
        "under 20260830T050000Z-root",
    ] {
        assert!(text.contains(word), "{word:?}:\n{text}");
    }
    assert!(text.contains("120 tokens"), "{text}");
}

/// **The seat's card** stands where nothing is selected: the version, each
/// server with what it holds, and the ways in — the conversation, the
/// model and the provider among them (bl-4e55).
#[test]
fn the_seat_s_card_stands_where_nothing_is_selected() {
    let mut model = Model {
        conversation: None,
        ..seated()
    };
    model.roster[0].workspaces[0].attention = 2;
    let text = painted(&mut model);
    for word in [
        version().as_str(),
        "(this box's own server) — 1 workspace, 2 conversations, 2 waiting",
        "Ctrl+X for a conversation",
        "Ctrl+O for the model",
        "/login for a provider",
        super::FOUNDING,
    ] {
        assert!(text.contains(word), "{word:?}:\n{text}");
    }
    let mut quiet = seated();
    let text = painted(&mut quiet);
    assert!(!text.contains(&version()), "{text}");
    assert!(WAYS.iter().all(|way| ways_in().contains(way)));
    let mut one = Model {
        conversation: None,
        ..seated()
    };
    one.roster[0].workspaces[0].agents = 1;
    let text = painted(&mut one);
    assert!(text.contains("1 workspace, 1 conversation"), "{text}");
}

/// The dial state on the card and the rule.
mod held;
/// The roster read refused: on the card and the switcher, never the bar.
mod refused;
