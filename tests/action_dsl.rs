//! Integration tests for the workflow action DSL parser.

use lernie::config::action::{Action, DispatchMode};

#[test]
fn parses_zero_arg_actions() {
    assert_eq!(
        Action::parse("spawn_exchange").unwrap(),
        Action::SpawnExchange
    );
    assert_eq!(
        Action::parse("spawn_root_conversation").unwrap(),
        Action::SpawnRootConversation
    );
    assert_eq!(Action::parse("merge").unwrap(), Action::Merge);
    assert_eq!(
        Action::parse("mark_abandoned").unwrap(),
        Action::MarkAbandoned
    );
    assert_eq!(Action::parse("notify_ui").unwrap(), Action::NotifyUi);
}

#[test]
fn parses_dispatch_role_only() {
    assert_eq!(
        Action::parse("dispatch(worker)").unwrap(),
        Action::Dispatch {
            role: "worker".into(),
            with: None,
            mode: None
        }
    );
}

#[test]
fn parses_dispatch_with_kwarg() {
    assert_eq!(
        Action::parse("dispatch(worker, with: verifier.feedback)").unwrap(),
        Action::Dispatch {
            role: "worker".into(),
            with: Some("verifier.feedback".into()),
            mode: None
        }
    );
}

#[test]
fn parses_dispatch_with_mode() {
    assert_eq!(
        Action::parse("dispatch(compactor, mode: intermediate)").unwrap(),
        Action::Dispatch {
            role: "compactor".into(),
            with: None,
            mode: Some(DispatchMode::Intermediate)
        }
    );
}

#[test]
fn parses_gate_merge_on() {
    assert_eq!(
        Action::parse("gate_merge_on(verifier.approve)").unwrap(),
        Action::GateMergeOn {
            predicate: "verifier.approve".into()
        }
    );
}

#[test]
fn rejects_unknown_action() {
    let e = Action::parse("teleport(worker)").unwrap_err();
    assert!(e.contains("unknown action"), "got: {e}");
}

#[test]
fn rejects_zero_arg_action_with_args() {
    let e = Action::parse("merge(now)").unwrap_err();
    assert!(e.contains("no arguments"), "got: {e}");
}

#[test]
fn rejects_dispatch_without_role() {
    assert!(Action::parse("dispatch()").is_err());
    assert!(Action::parse("dispatch(with: x)").is_err());
}

#[test]
fn rejects_dispatch_unknown_kwarg() {
    let e = Action::parse("dispatch(worker, retries: 3)").unwrap_err();
    assert!(e.contains("unknown named arg"), "got: {e}");
}

#[test]
fn rejects_dispatch_unknown_mode() {
    let e = Action::parse("dispatch(worker, mode: terminal)").unwrap_err();
    assert!(e.contains("unknown mode"), "got: {e}");
}

#[test]
fn rejects_dispatch_extra_positional() {
    let e = Action::parse("dispatch(worker, extra)").unwrap_err();
    assert!(e.contains("at most one positional"), "got: {e}");
}

#[test]
fn rejects_gate_merge_on_arity() {
    assert!(Action::parse("gate_merge_on()").is_err());
    assert!(Action::parse("gate_merge_on(a, b)").is_err());
}

#[test]
fn rejects_unbalanced_parens() {
    assert!(Action::parse("dispatch(worker").is_err());
}

#[test]
fn rejects_invalid_identifier() {
    assert!(Action::parse("bad-action").is_err());
}

#[test]
fn rejects_empty_argument() {
    assert!(Action::parse("dispatch(worker, , mode: intermediate)").is_err());
}

#[test]
fn rejects_invalid_value() {
    assert!(Action::parse("dispatch(worker!)").is_err());
    assert!(Action::parse("dispatch(worker, with: bad value)").is_err());
}

#[test]
fn rejects_invalid_kwarg_key() {
    assert!(Action::parse("dispatch(worker, bad-key: x)").is_err());
}

#[test]
fn rejects_empty_action_name() {
    let e = Action::parse("(worker)").unwrap_err();
    assert!(e.contains("empty identifier"), "got: {e}");
}

#[test]
fn rejects_empty_kwarg_value() {
    let e = Action::parse("dispatch(worker, mode: )").unwrap_err();
    assert!(e.contains("empty value"), "got: {e}");
}
