//! **What the field says while it asks** (bl-1252; bl-0531): the hint
//! `ask::keys` lends the prompt line, worded from the cells on offer.

use crate::test_support::window::{asked, painted};

/// **The field's hint while it asks names the digits and the words**
/// (bl-1252): `1–6` where a call is parked, what the one key does where only
/// `seen` is offered (bl-0531), and no digits before the queue's row has
/// landed.
#[test]
fn the_hint_names_the_digits_and_the_words() {
    let mut model = asked();
    assert_eq!(
        super::super::hint(&model),
        "it is asking — 1–6 answers, or say it — Enter sends"
    );
    for section in &mut model.waiting {
        for row in &mut section.rows {
            row.held = None;
        }
    }
    assert_eq!(
        super::super::hint(&model),
        "it is asking — Enter or 1 marks it seen, or say it — Enter sends"
    );
    model.waiting.clear();
    assert_eq!(
        super::super::hint(&model),
        "it is asking — say it — Enter sends"
    );
    let text = painted(&mut asked());
    assert!(text.contains("1–6 answers"), "{text}");
}
