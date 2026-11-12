//! The register page's state: when it fires, and what a claim's return does.

use super::{Phase, Registering};
use crate::seat::register::Registered;
use crate::test_support::window::seated;

fn registered() -> Registered {
    Registered {
        workspace: "lab".to_owned(),
        name: "laptop".to_owned(),
        grade: "operator".to_owned(),
        word: "K7M2QX".to_owned(),
        entry: std::path::PathBuf::from("/home/u/.local/share/lernie/wire/workspaces/lab"),
        addresses: vec![
            "engine.example:7737".to_owned(),
            "192.0.2.7:7737".to_owned(),
        ],
        proved: Ok(()),
    }
}

/// **Nothing typed fires nothing, and a claim that is out is not fired
/// twice.** What fires is the URL trimmed, handed over once.
#[test]
fn the_claim_fires_once_for_what_is_typed() {
    let mut model = seated();
    assert!(!model.register(), "nothing typed");
    model.registering.url = "   ".to_owned();
    assert!(!model.register(), "only space typed");
    model.registering.url = " https://engine.example/yog/register/ABCD2345\n".to_owned();
    assert!(model.register());
    assert_eq!(
        model.registering.asked.as_deref(),
        Some("https://engine.example/yog/register/ABCD2345")
    );
    assert_eq!(model.registering.phase, Phase::Claiming);
    assert!(!model.register(), "one at a time");
}

/// **A box that registered holds one more channel**: the roster gains its
/// section, dialling the list the entry states, and the spent URL is cleared.
/// A second return for the same workspace seats nothing twice.
#[test]
fn a_claim_that_succeeded_seats_the_new_channel_and_clears_the_field() {
    let mut model = seated();
    let before = model.roster.len();
    model.registering.url = "https://engine.example/yog/register/ABCD2345".to_owned();
    model.register();
    model.registered(Ok(registered()));
    assert_eq!(model.registering.phase, Phase::Done(Ok(registered())));
    assert_eq!(model.registering.url, "");
    let seated = model.roster.last().expect("a section");
    assert_eq!(seated.channel.name, "lab");
    assert_eq!(seated.channel.named_there.as_deref(), Some("lab"));
    assert_eq!(
        seated.channel.dials.as_deref(),
        Some("engine.example:7737, 192.0.2.7:7737")
    );
    model.registered(Ok(registered()));
    assert_eq!(model.roster.len(), before + 1);
}

/// **A claim that failed keeps the URL** for the operator to read against
/// the refusal, seats nothing, and lets the button fire again.
#[test]
fn a_claim_that_failed_keeps_the_url_and_can_be_fired_again() {
    let mut model = seated();
    let before = model.roster.len();
    model.registering.url = "https://engine.example/yog/register/ABCD2345".to_owned();
    model.register();
    model.registered(Err("this offer cannot be claimed".to_owned()));
    assert_eq!(model.roster.len(), before);
    assert!(!model.registering.url.is_empty());
    assert!(model.registering.ready());
    assert_eq!(Registering::default().phase, Phase::Idle);
}
