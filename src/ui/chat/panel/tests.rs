//! The two panels: the conversation's facts over its transcript, and the
//! seat's where nothing is selected.

use super::{WAYS_IN, version};
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
        "on home — (this box's own engine)",
        "under 20260830T050000Z-root",
    ] {
        assert!(text.contains(word), "{word:?}:\n{text}");
    }
    assert!(text.contains("120 tokens"), "{text}");
}

/// **The seat's card** stands where nothing is selected: the version, each
/// engine with what it holds, and the two ways in.
#[test]
fn the_seat_s_card_stands_where_nothing_is_selected() {
    let mut model = Model {
        conversation: None,
        ..seated()
    };
    model.roster[0].walls[0].attention = 2;
    let text = painted(&mut model);
    for word in [
        version().as_str(),
        "(this box's own engine) — 1 wall, 2 conversations, 2 waiting",
        WAYS_IN,
    ] {
        assert!(text.contains(word), "{word:?}:\n{text}");
    }
    let mut quiet = seated();
    let text = painted(&mut quiet);
    assert!(!text.contains(WAYS_IN), "{text}");
    let mut one = Model {
        conversation: None,
        ..seated()
    };
    one.roster[0].walls[0].agents = 1;
    let text = painted(&mut one);
    assert!(text.contains("1 wall, 1 conversation"), "{text}");
}
