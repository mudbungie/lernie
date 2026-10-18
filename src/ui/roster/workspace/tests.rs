//! One workspace's row: the line it wears, and the pin that follows the row's own
//! rank (its control is a row of the menu now, `crate::ui::slash`).

use crate::reply::roster::WsRow;
use crate::test_support::window::workspace;
use crate::ui::Model;

/// **A seat aimed at nothing composes no pin**, whichever way it is asked.
#[test]
fn an_aimless_seat_composes_no_pin() {
    let mut model = Model::default();
    model.post_pin(true);
    assert!(model.outbox.is_empty());
}

/// A row this seat cannot address is painted as what it is, and carries no
/// control at all.
#[test]
fn a_row_with_no_entry_naming_it_wears_the_line_and_no_control() {
    let quiet = super::line(&workspace("home"));
    assert!(quiet.contains("home"), "{quiet}");
    let busy = super::line(&WsRow {
        attention: 3,
        running: true,
        ..workspace("home")
    });
    assert!(
        busy.contains("3 waiting") && busy.contains("running"),
        "{busy}"
    );
}

/// **A workspace holding one conversation says `1 conversation`** (bl-967f), as
/// the seat's card above it does: one helper words every count.
#[test]
fn one_conversation_is_singular() {
    let mut one = workspace("home");
    one.agents = 1;
    assert!(
        super::line(&one).contains("1 conversation  "),
        "{}",
        super::line(&one)
    );
    one.agents = 2;
    assert!(super::line(&one).contains("2 conversations"));
    assert_eq!(crate::render::counted(0, "workspace"), "0 workspaces");
    assert_eq!(crate::render::counted(1, "line"), "1 line");
}
