//! The two walkers, against shapes the vendored corpus does not have. At the
//! cut every stamp is at or below the floor, so a projection over real frames
//! deletes nothing — every arm below would otherwise be an arm nobody has
//! evidence about.

use serde_json::json;

use super::super::Signature;
use super::{mutate, project, spelled, words};

/// A signature, written the way upstream writes one: `<path>:<type>` to the
/// edition the path appeared at.
fn signature(entries: &[(&str, u64)]) -> Signature {
    entries
        .iter()
        .map(|(path, stamp)| ((*path).to_owned(), *stamp))
        .collect()
}

/// A signature entry is a path and the JSON type it is spelled with, and a
/// nullable field is spelled twice — once per type.
#[test]
fn a_signature_entry_is_a_path_and_a_type() {
    assert_eq!(spelled("/rows/[]/held:null"), ("/rows/[]/held", "null"));
    assert_eq!(spelled(":object"), ("", "object"));
    assert_eq!(spelled("nonsense"), ("nonsense", ""));
}

/// **The vocabulary paths are the string-typed ones, and never `kind`.** The
/// discriminant stays strict by name, so mutating it would assert the opposite
/// of the rule (DESIGN §4.9, rung 2 against rung 3).
#[test]
fn the_word_paths_are_the_string_ones_without_the_discriminant() {
    let signature = signature(&[
        ("/kind:string", 1),
        ("/rows/[]/state:string", 1),
        ("/rows/[]/age:number", 1),
        ("/rows:array", 1),
        (":object", 1),
    ]);
    assert_eq!(words(&signature), vec!["/rows/[]/state".to_owned()]);
}

/// **Projection deletes what a stamp says the engine could not write**, at
/// every depth and through every element, and leaves everything at or below
/// the asked-for edition exactly as it was.
#[test]
fn projection_deletes_only_what_is_stamped_later() {
    let signature = signature(&[
        ("/kind:string", 1),
        ("/rows/[]/says:string", 1),
        ("/rows/[]/held:object", 20),
        ("/rows/[]/held/tool:string", 20),
    ]);
    let frame = json!({
        "kind": "follow",
        "rows": [
            {"says": "one", "held": {"tool": "Bash"}},
            {"says": "two"},
        ],
    });
    assert_eq!(
        project(&frame, &signature, 20),
        frame,
        "nothing is later than 20"
    );
    assert_eq!(
        project(&frame, &signature, 19),
        json!({"kind": "follow", "rows": [{"says": "one"}, {"says": "two"}]}),
    );
}

/// **A container's element spelling projects to the empty array**, which is the
/// container's own path saying the same thing one level down.
#[test]
fn projecting_an_element_spelling_empties_the_array() {
    let signature = signature(&[("/rows/[]/signals/[]:string", 20)]);
    let frame = json!({"rows": [{"signals": ["flagged"]}, {"signals": []}]});
    assert_eq!(
        project(&frame, &signature, 19),
        json!({"rows": [{"signals": []}, {"signals": []}]}),
    );
}

/// A path that runs off this particular frame reaches nothing and deletes
/// nothing: a shape spells the union of what its frames hold.
#[test]
fn projecting_a_path_this_frame_does_not_carry_changes_nothing() {
    let signature = signature(&[
        ("/rows/[]/held/tool:string", 20),
        ("/absent/deep:string", 20),
    ]);
    let frame = json!({"rows": [{"says": "one"}]});
    assert_eq!(project(&frame, &signature, 19), frame);
}

/// **Mutation replaces the word wherever the path reaches a string**, and
/// answers `None` where it reaches none — including where the path reaches a
/// value of another type entirely.
#[test]
fn mutation_sets_every_string_the_path_reaches() {
    let frame = json!({
        "rows": [
            {"state": "running", "signals": ["flagged", "truncated"], "age": 3},
            {"state": "wound"},
        ],
    });
    assert_eq!(
        mutate(&frame, "/rows/[]/state", "unheard-of"),
        Some(json!({
            "rows": [
                {"state": "unheard-of", "signals": ["flagged", "truncated"], "age": 3},
                {"state": "unheard-of"},
            ],
        })),
    );
    assert_eq!(
        mutate(&frame, "/rows/[]/signals/[]", "unheard-of"),
        Some(json!({
            "rows": [
                {"state": "running", "signals": ["unheard-of", "unheard-of"], "age": 3},
                {"state": "wound"},
            ],
        })),
    );
    assert_eq!(mutate(&frame, "/rows/[]/absent", "unheard-of"), None);
    assert_eq!(mutate(&frame, "/rows/[]/age", "unheard-of"), None);
    assert_eq!(mutate(&frame, "", "unheard-of"), None);
}
