//! **The bar's own worlds** (DESIGN §4.44): every other world photographs it
//! shut, so the open bar and the workspace's list at its foot are screens the
//! walk visits only here — the parity instrument's reason, one pane over.

use super::World;
use crate::test_support::window::seated;

/// **The bar pinned open** over a seated window: its rows in full, the pin
/// saying `unpin`, `+ new` at its head and the wall at its foot.
pub(super) fn opened() -> World {
    let mut model = seated();
    model.bar.pinned = true;
    World { name: "bar", model }
}

/// **The workspace's list standing upward from the bar's foot**, the one
/// screen `+ new workspace` is on outside the switcher.
pub(super) fn footed() -> World {
    let mut model = seated();
    model.drop_up(crate::ui::Selector::Workspace);
    World {
        name: "workspaces",
        model,
    }
}
