//! Every ink, and the one rule that governs the unknown ones.

use super::{state_ink, tone_ink, visuals};
use crate::reply::convs::{AgentState, Tone};

/// **Rung 3, on the glass**: a token this build does not know is unstyled, not
/// restyled as a token it is not. A colour is a claim, and the wrong claim is
/// worse than none.
#[test]
fn an_unknown_token_paints_plain_and_never_borrows_another_word_s_colour() {
    let plain = state_ink(&AgentState::Quiescent);
    assert_eq!(state_ink(&AgentState::Unknown("parked".to_owned())), plain);
    for known in [AgentState::Live, AgentState::InFlight, AgentState::Stopped] {
        assert_ne!(state_ink(&known), plain, "{known:?} is styled");
    }
    let flat = tone_ink(&Tone::Plain);
    assert_eq!(tone_ink(&Tone::Unknown("amber".to_owned())), flat);
    for known in [
        Tone::Live,
        Tone::InFlight,
        Tone::Good,
        Tone::Bad,
        Tone::Weak,
    ] {
        assert_ne!(tone_ink(&known), flat, "{known:?} is styled");
    }
}

/// Dark, because a seat is looked at for hours beside a terminal that already
/// is.
#[test]
fn the_window_is_dark() {
    assert!(visuals().dark_mode);
}
