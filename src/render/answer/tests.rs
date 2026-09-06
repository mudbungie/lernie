//! The dispatch's own arm: a kind is rendered by the family it belongs to, and
//! the receipts that carry one fact are sentences rather than listings.

use serde_json::json;

use crate::render::rendered;

/// The receipts, each in one sentence and each saying what the frame says and
/// nothing it would have to predict.
#[test]
fn every_receipt_is_the_one_sentence_its_frame_carries() {
    for (frame, expected) in [
        (json!({"ok": true, "kind": "nudged"}), "nudged"),
        (json!({"ok": true, "kind": "flagged"}), "flagged"),
        (json!({"ok": true, "kind": "acked"}), "watermark"),
        (json!({"ok": true, "kind": "trail-cleared"}), "truncated"),
        (
            json!({"ok": true, "kind": "started", "conversation": "PelicanQuiet"}),
            "started PelicanQuiet",
        ),
        (
            json!({"ok": true, "kind": "marks", "branch": "balls/tasks"}),
            "balls/tasks",
        ),
    ] {
        let said = rendered(&frame);
        assert!(said.contains(expected), "{frame}: {said}");
    }
}
