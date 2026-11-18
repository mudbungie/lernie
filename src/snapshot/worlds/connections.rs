//! **The connections page's world** (DESIGN §4.52): this box's own server,
//! an entry dialling the same address, and one armed for removal — the
//! page's every shape on one screen.

use super::World;
use crate::test_support::window::seated;
use crate::ui::{Channel, Chunk, Held, Lookup, Page};

/// *Settings › Servers › connections* with a duplicate and an armed removal.
pub(super) fn connections() -> World {
    let mut model = seated();
    model.roster[0].channel.dials = Some("127.0.0.1:7737".to_owned());
    for (name, dials, held) in [
        ("lab", "127.0.0.1:7737", Held::Heard),
        ("zoo", "engine.example:7737", Held::Unheard),
    ] {
        model.roster.push(Chunk {
            channel: Channel {
                name: name.to_owned(),
                named_there: Some(name.to_owned()),
                dials: Some(dials.to_owned()),
            },
            held,
            ..Chunk::default()
        });
    }
    model.arm_forget("lab");
    model.lookup = Some(Lookup::Settings(Page::Connections));
    World {
        name: "connections",
        model,
    }
}
