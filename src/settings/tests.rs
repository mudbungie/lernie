//! The table's invariants, and the two projections round-tripping.

use super::{BAR_SHUT, DESK, KNOBS, Kind, Knobbed, Settings, Toggle, from_json, knob, to_json};
use serde_json::{Value, json};

/// **Every knob has its own key, its own label and a sentence**, and every
/// default stands inside its bounds — the table judged once, so a row added
/// badly fails here and not on the pane.
#[test]
fn every_knob_is_distinct_and_its_default_is_inside_its_bounds() {
    let rest = Settings::default();
    let mut keys = std::collections::BTreeSet::new();
    let mut labels = std::collections::BTreeSet::new();
    for row in KNOBS {
        assert!(keys.insert(row.key), "{} twice", row.key);
        assert!(labels.insert(row.label), "{} twice", row.label);
        assert!(!row.help.is_empty(), "{} says nothing", row.key);
        match (row.kind, (row.get)(&rest)) {
            (Kind::Toggle, Knobbed::On(_)) => {}
            (
                Kind::Number {
                    floor,
                    ceiling,
                    step,
                    unit,
                },
                Knobbed::At(at),
            ) => {
                assert!(floor <= at && at <= ceiling, "{} rests at {at}", row.key);
                assert!(step > 0.0 && floor < ceiling, "{}", row.key);
                assert!(!unit.is_empty(), "{} has no unit", row.key);
            }
            (kind, value) => panic!("{} is {kind:?} but reads {value:?}", row.key),
        }
    }
    assert_eq!(KNOBS.len(), 12, "DESIGN §4.46 item 4's table");
    // **The two shapes the rows are spelled with**, run rather than only
    // folded into the table at compile time.
    assert_eq!(
        super::seconds(3.0, 0.1),
        Kind::Number {
            unit: " s",
            floor: 0.0,
            ceiling: 3.0,
            step: 0.1
        }
    );
    assert_eq!(
        super::points(1.0, 2.0, 0.5),
        Kind::Number {
            unit: " pt",
            floor: 1.0,
            ceiling: 2.0,
            step: 0.5
        }
    );
    assert!(knob("wheel").is_some());
    assert!(
        knob("pinned").is_none(),
        "the pin is the place file's own key"
    );
}

/// **A write of the other kind writes nothing**: the accessors are typed by
/// the row, and a toggle handed a number — or a number a toggle — keeps what
/// it held.
#[test]
fn a_value_of_the_other_kind_writes_nothing() {
    let rest = Settings::default();
    for row in KNOBS {
        let mut held = Settings::default();
        let wrong = match row.kind {
            Kind::Toggle => Knobbed::At(1.0),
            Kind::Number { .. } => Knobbed::On(false),
        };
        (row.set)(&mut held, wrong);
        assert_eq!(held, rest, "{}", row.key);
    }
}

/// **The two widths read as the policy's defaults until set**, and a set
/// writes the number (DESIGN §4.45 item 2 moved under the table).
#[test]
fn the_widths_read_as_their_defaults_until_set() {
    let mut held = Settings::default();
    let shut = knob("bar_shut").expect("a knob");
    let open = knob("bar_open").expect("a knob");
    assert_eq!((shut.get)(&held), Knobbed::At(BAR_SHUT));
    assert_eq!((open.get)(&held), Knobbed::At(DESK / 4.0));
    (shut.set)(&mut held, Knobbed::At(64.5));
    (open.set)(&mut held, Knobbed::At(420.0));
    assert_eq!(held.bar_shut, Some(64.5));
    assert_eq!(held.bar_open, Some(420.0));
}

/// **Nothing set writes nothing, and everything set comes back**: the file
/// carries only what stands off its default.
#[test]
fn only_what_is_off_its_default_is_written_and_all_of_it_comes_back() {
    assert_eq!(to_json(&Settings::default()), json!({}));
    let set = Settings {
        wheel: 200.0,
        help_delay: 1.5,
        scale: 1.25,
        linger: 2.0,
        ease: 0.0,
        bar_shut: Some(64.5),
        bar_open: Some(420.0),
        on: [Toggle::Dock].into(),
    };
    let written = to_json(&set);
    assert_eq!(written.as_object().map(serde_json::Map::len), Some(12));
    assert_eq!(from_json(&written), set);
    let one = Settings {
        linger: 2.0,
        ..Settings::default()
    };
    assert_eq!(to_json(&one), json!({ "linger": 2.0 }));
    assert_eq!(from_json(&to_json(&one)), one);
}

/// **Every way the object can be wrong is the default, per knob**: absent,
/// null, the wrong shape, a key nobody answers to — and the knobs beside the
/// wrong one are still read.
#[test]
fn every_wrong_value_is_the_default_and_costs_only_itself() {
    let rest = Settings::default();
    assert_eq!(from_json(&Value::Null), rest);
    assert_eq!(from_json(&json!("wide")), rest);
    assert_eq!(from_json(&json!({ "roster_width": 360.5 })), rest);
    let mixed = json!({
        "wheel": "fast",
        "help": null,
        "help_delay": true,
        "fold": 0,
        "dock": true,
        "ease": 0.5
    });
    let mut expected = Settings { ease: 0.5, ..rest };
    expected.set(Toggle::Dock, true);
    assert_eq!(from_json(&mixed), expected);
}

/// **The switches are a set and the query reads it**: four on by default,
/// and a set turns one on or off without touching the rest.
#[test]
fn the_switches_are_a_set_and_the_query_reads_it() {
    let mut held = Settings::default();
    assert!(held.on(Toggle::Help) && held.on(Toggle::Confirm));
    assert!(!held.on(Toggle::Dock));
    held.set(Toggle::Dock, true);
    held.set(Toggle::Help, false);
    assert!(held.on(Toggle::Dock) && !held.on(Toggle::Help));
    assert!(held.on(Toggle::Bubbles) && held.on(Toggle::Fold));
}
