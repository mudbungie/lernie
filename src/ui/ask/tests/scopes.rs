//! **The reaches `pass` is offered at follow the held row** (yog bl-0da6;
//! bl-da16): the rows, the digits that number them, the hint that counts
//! them, and the queue pane reading the same list.

use super::super::{Choice, choices, pick, row};
use crate::paint_probe::frame::Window;
use crate::test_support::window::asked;
use crate::ui::Model;
use serde_json::json;

/// The asking model, its held row listing `scopes`.
fn listing(scopes: &[&str]) -> Model {
    let mut model = asked();
    let held = model.waiting[0].rows[0]
        .held
        .as_mut()
        .expect("a parked call");
    held.scopes = scopes.iter().map(|s| (*s).to_owned()).collect();
    model
}

/// The words on the rows a model's selected conversation is offered.
fn words(model: &Model) -> Vec<String> {
    let (_, held) = row(model).expect("a row");
    choices(&held).into_iter().map(Choice::word).collect()
}

/// **A row that lists `call` alone offers no wider pass**: a secret or
/// destructive call, which the server answers for the call in front of the
/// operator and refuses at any other reach — so the rows are the four that
/// go through, numbered as four, with the digits and the hint counting the
/// same.
#[test]
fn a_row_that_lists_call_alone_offers_no_wider_pass() {
    let mut model = listing(&["call"]);
    assert_eq!(words(&model), vec!["seen", "pass", "refuse", "hold"]);
    assert_eq!(
        super::super::hint(&model),
        "it is asking — 1–4 answers, or say it — Enter sends"
    );
    // Two frames, because a bottom panel sizes from the last one.
    let window = Window::new();
    window.text(|ctx| crate::ui::render(ctx, &mut model));
    let text = window.text(|ctx| crate::ui::render(ctx, &mut model));
    for said in ["1. seen", "2. pass", "3. refuse", "4. hold", "1–4 answers"] {
        assert!(text.contains(said), "{said:?}:\n{text}");
    }
    assert!(!text.contains("for this"), "{text}");
    assert!(pick(&mut model, 4));
    assert_eq!(
        model.outbox.last().map(|p| p.envelope["verdict"].clone()),
        Some(json!("hold"))
    );
    assert_eq!(
        model.outbox.last().map(|p| p.envelope["scope"].clone()),
        Some(json!("call"))
    );
    assert!(!pick(&mut model, 5), "no fifth row");
}

/// **A row that lists all three offers the five verdicts**, in the wire's
/// order whatever order the row said them in; **a row that lists none is an
/// older server, and offers the same**; **a word this build cannot spell is
/// not offered**, and the ones it can are.
#[test]
fn the_listed_reaches_are_offered_in_the_wire_s_order_and_only_the_known_ones() {
    let five = vec![
        "seen",
        "pass",
        "pass for this conversation",
        "pass for this workspace",
        "refuse",
        "hold",
    ];
    assert_eq!(
        words(&listing(&["workspace", "call", "conversation"])),
        five
    );
    assert_eq!(words(&listing(&[])), five);
    assert_eq!(
        words(&listing(&["call", "everywhere"])),
        vec!["seen", "pass", "refuse", "hold"]
    );
    assert_eq!(
        words(&listing(&["everywhere", "workspace"])),
        vec!["seen", "pass for this workspace", "refuse", "hold"]
    );
    assert!(Choice::Verdict("pass", "workspace").wider());
    assert!(!Choice::Verdict("refuse", "call").wider());
    assert!(!Choice::Seen.wider());
}

/// **Every choice says what it does to the call** (DESIGN §4.46 item 2),
/// and a verdict wider than the call says so.
#[test]
fn every_choice_tells_and_a_wide_verdict_names_its_scope() {
    let call = crate::verbs::capability::CALL;
    assert!(Choice::Seen.help().contains("parked"));
    assert!(Choice::Verdict("pass", call).help().contains("lets"));
    assert!(Choice::Verdict("refuse", call).help().contains("refuses"));
    assert!(Choice::Verdict("hold", call).help().contains("parked"));
    let wide = Choice::Verdict("pass", "workspace").help();
    assert!(
        wide.contains("every call like it in this workspace"),
        "{wide}"
    );
}
