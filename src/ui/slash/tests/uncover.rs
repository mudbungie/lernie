//! **One overlay stands at a time, from under anything** (bl-8c80): a word
//! fired while the switcher or a pane stands puts that down first.

use super::super::fire;
use crate::test_support::window::seated;
use crate::ui::Lookup;

/// **A pane opened by its word from under the switcher stands alone.**
#[test]
fn a_word_fired_under_the_switcher_puts_the_switcher_down() {
    let mut model = seated();
    model.switch();
    assert!(model.switching);
    assert!(fire(&mut model, crate::ui::settings::WORD, ""));
    assert!(!model.switching, "the switcher is down");
    assert_eq!(
        model.lookup,
        Some(Lookup::Settings(crate::ui::Page::default()))
    );
}

/// **A pane opened by its word from under another pane stands alone**, the
/// other put down by the steps Esc would take.
#[test]
fn a_word_fired_under_a_pane_puts_the_pane_down() {
    let mut model = seated();
    model.begin_tuning();
    assert!(model.tuning.is_some());
    assert!(fire(&mut model, crate::ui::commands::WORD, ""));
    assert!(model.tuning.is_none(), "the tuning pane is down");
    assert!(model.commanding());
}
