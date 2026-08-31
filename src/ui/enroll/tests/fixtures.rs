//! The enrollment pane's fixtures, shared by both halves of its suite: the
//! fabricated material, the frame it really arrives in, and the door it comes
//! through.

use serde_json::json;

use crate::reply::enrolled::Enrolled;
use crate::test_support::window::seated;
use crate::ui::Model;

/// Fabricated material, `notreal` throughout — and the key deliberately carries
/// no private-key banner, because the disclosure gate reads every committed
/// byte of this tree.
pub(super) fn material() -> Enrolled {
    Enrolled {
        grade: "foot".to_owned(),
        name: "phone-1".to_owned(),
        address: "engine.invalid:7737".to_owned(),
        ca: "-----BEGIN CERTIFICATE-----\nnotreal-ca\n-----END CERTIFICATE-----\n".to_owned(),
        cert: "-----BEGIN CERTIFICATE-----\nnotreal-leaf\n-----END CERTIFICATE-----\n".to_owned(),
        key: "-----BEGIN notreal KEY-----\nnotreal-key\n-----END notreal KEY-----\n".to_owned(),
    }
}

/// The material as a wire frame, which is how it actually arrives.
fn frame() -> serde_json::Value {
    let held = material();
    json!({
        "ok": true, "kind": "enrolled",
        "grade": held.grade, "name": held.name, "address": held.address,
        "ca": held.ca, "cert": held.cert, "key": held.key,
    })
}

/// **File the material the way the window really does** — through
/// [`Model::absorb`], the one door, rather than through the arm behind it. A
/// test that reached past the door would pass while the door was unwired.
pub(super) fn file(model: &mut Model) {
    model.absorb(&crate::ui::Channel::default(), crate::reply::read(&frame()));
}

/// A window aimed at a wall with an enrollment open on it.
pub(super) fn opened() -> Model {
    let mut model = seated();
    model.begin_enrollment();
    model
}
