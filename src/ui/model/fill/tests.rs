//! The navigation's two doors: what it moves, what it does not, and that the
//! request is taken rather than read.

use super::Fill;
use crate::test_support::window::seated;
use crate::ui::Column;

/// **It names the conversation, goes where the box is, and asks once.** All
/// three matter: the composer acts on the selection, the narrow shape paints
/// the composer on the conversation's own column and nowhere else, and a
/// request that survived the frame answering it would drag the cursor back.
#[test]
fn going_to_a_box_selects_the_row_moves_to_its_column_and_is_taken_once() {
    let mut model = seated();
    model.column = Column::Channels;
    model.fill_in("another conversation", Fill::Reason);
    assert_eq!(model.conversation.as_deref(), Some("another conversation"));
    assert_eq!(model.column, Column::Conversation);
    assert_eq!(model.filling(), Some(Fill::Reason));
    assert_eq!(model.filling(), None, "taken, not read");
}

/// **It composes nothing**, which is the whole of the row menu's answer to
/// §4.20: the item that would unmake a conversation opens the arming instead
/// of firing, so a mis-aimed click in a list cannot destroy anything.
#[test]
fn a_navigation_composes_no_gesture_at_all() {
    let mut model = seated();
    model.fill_in("20260830T051200Z-a1b2", Fill::Arming);
    assert!(
        model.outbox.is_empty(),
        "going to a box asks for nothing: {:?}",
        model.outbox
    );
    assert_eq!(model.filling(), Some(Fill::Arming));
}
