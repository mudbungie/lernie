//! The two envelopes: what they carry, and the one field the fire rewrites.

use super::{prepare, prompt};
use crate::reply::start::Prepared;
use serde_json::json;

/// A staged body as an engine answers one, with a field this build does not
/// read in it.
fn staged(workspace: &str) -> Prepared {
    let body = json!({"workspace": workspace, "goal": "",
                      "binding": null, "lineage": null, "origin": "world"});
    Prepared {
        workspace: workspace.to_owned(),
        goal: String::new(),
        body,
    }
}

/// **The bare rung, spelled out.** The payload is the gesture's own and the
/// address is top level, so [`crate::seat::route`] resolves it exactly as it
/// resolves a read's.
#[test]
fn staging_names_the_workspace_and_the_bare_rung() {
    assert_eq!(
        prepare("home".to_owned(), None),
        json!({"op": "prepare", "workspace": "home", "payload": {"rung": "bare"}})
    );
}

/// **The staged body crosses back verbatim**, with the workspace re-addressed
/// into this box's spelling — and nothing else touched, including the fields
/// this build never read.
#[test]
fn the_fire_hands_the_body_back_whole_and_re_addressed() {
    let fired = prompt(
        &staged("personal"),
        "home".to_owned(),
        "do it".to_owned(),
        None,
    );
    assert_eq!(
        fired,
        json!({"op": "prompt", "goal": "do it", "seed": null,
               "prepared": {"workspace": "home", "goal": "",
                            "binding": null, "lineage": null, "origin": "world"}})
    );
}

/// **The re-addressing is what makes the second act reach the first's engine.**
/// The body comes back in the host's spelling; §8.2's mapping runs client→host
/// at the channel boundary and nowhere else, so a body handed back unrewritten
/// names a workspace no entry claims and the fire falls through to this box's
/// own engine.
#[test]
fn the_fire_is_addressed_where_the_seat_can_route_it() {
    let fired = prompt(
        &staged("personal"),
        "home".to_owned(),
        "do it".to_owned(),
        None,
    );
    assert_eq!(
        crate::envelope::workspace(&fired),
        Some("home".to_owned()),
        "the envelope's workspace is the one the router reads"
    );
}

/// The two `op` words are the boundary's, and they are read off the envelopes
/// rather than restated: a rename upstream fails here rather than on the wire.
#[test]
fn both_envelopes_wear_the_boundary_s_own_op() {
    assert_eq!(prepare(String::new(), None)["op"], json!(super::PREPARE));
    assert_eq!(
        prompt(&staged("home"), String::new(), String::new(), None)["op"],
        json!(super::PROMPT)
    );
}

/// **The path rung, spelled out** (bl-4371). The rung word and the target are
/// two fields of one payload, so a `dir` cannot be sent without saying which
/// rung it belongs to — upstream's own rule that the rung is said and never
/// inferred.
#[test]
fn staging_with_a_work_target_names_the_path_rung_and_the_directory() {
    assert_eq!(
        prepare("home".to_owned(), Some("/work/repo".to_owned())),
        json!({"op": "prepare", "workspace": "home",
               "payload": {"rung": "path", "dir": "/work/repo"}})
    );
}

/// **The fire's goal is the rung's prefill and then the operator's.** The bare
/// rung prefills nothing and the goal is untouched; the path rung prefills the
/// target preamble, and a fire that dropped it would carry a conversation bound
/// to a directory it was never told about.
#[test]
fn a_rung_with_a_prefill_fires_it_ahead_of_what_was_typed() {
    assert_eq!(super::goal("", "do the thing"), "do the thing");
    assert_eq!(super::goal("   \n ", "do the thing"), "do the thing");
    assert_eq!(
        super::goal("Working directory: /w\nDo all work there.", "do the thing"),
        "Working directory: /w\nDo all work there.\n\ndo the thing"
    );
}

/// **The role is the one field the seat writes into the body it carries back**
/// (REMOTE §9.21, PROTOCOL 18). `prepare` answers `null` — litany's `worker` —
/// so a fire that names no role is byte-identical to the fire this seat sent
/// before the field existed, and one that names a role states it on the body
/// that was going anyway rather than in a second gesture.
#[test]
fn a_role_is_written_onto_the_body_and_unstated_leaves_what_the_engine_said() {
    let bare = prompt(
        &staged("personal"),
        "home".to_owned(),
        "do it".to_owned(),
        None,
    );
    assert_eq!(bare["prepared"].get(super::ROLE), None);
    let planning = prompt(
        &staged("personal"),
        "home".to_owned(),
        "do it".to_owned(),
        Some("planner".to_owned()),
    );
    assert_eq!(planning["prepared"][super::ROLE], json!("planner"));
    // Everything else about the body is untouched, the re-addressing included.
    assert_eq!(planning["prepared"]["workspace"], json!("home"));
    assert_eq!(planning["prepared"]["origin"], json!("world"));
}
