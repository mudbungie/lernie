//! The conversation's panel over its transcript, and where the seat's facts
//! went when its card did (DESIGN §4.48 item 1).

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

/// **The seat's facts stand at the status bar's right end** (DESIGN §4.48
/// item 1), selected or not: the version and the queue's door while
/// anything asks — and the card that carried them, with its per-server
/// counts, is gone from the empty column.
#[test]
fn the_seat_s_facts_stand_on_the_status_bar_and_no_card_does() {
    let version = crate::cli::version();
    for conversation in [None, seated().conversation] {
        let mut model = Model {
            conversation,
            ..seated()
        };
        model.roster[0].workspaces[0].attention = 2;
        let text = painted(&mut model);
        for word in [version.as_str(), "2 waiting"] {
            assert!(text.contains(word), "{word:?}:\n{text}");
        }
        assert!(!text.contains("1 workspace,"), "{text}");
    }
}

/// The dial state at the status bar's right end.
mod held;
/// The roster read refused: on the status bar and the switcher.
mod refused;
