//! The asker's pass: the union across channels, the nesting, and the channel
//! that costs only itself.

use std::time::Duration;

use crate::state::{Link, Said};
use crate::ui::Model;
use serde_json::json;

/// A link whose standing set is what this model implies.
pub(super) fn asking(model: &Model) -> Link {
    let link = Link::new(Duration::from_millis(1));
    let mut model = model.clone();
    link.settle(&mut model);
    link
}

/// Everything one pass reported, in order.
pub(super) fn reported(link: &Link) -> Vec<(String, Said)> {
    let mut model = Model::default();
    let mut out = Vec::new();
    // `settle` is the only drain, so the frames are read back through it — the
    // same door the window uses, which is what keeps this a test of the pass
    // rather than of a private queue.
    link.settle(&mut model);
    out.extend(
        model
            .notice
            .map(|n| (String::new(), Said::Unreachable(n.line()))),
    );
    out.extend(
        model
            .roster
            .into_iter()
            .map(|chunk| (chunk.channel.name, Said::Frame(json!(chunk.walls.len())))),
    );
    out
}

/// The ball pane's four, which stand together at two widths from one pane.
mod board;
/// The machines, which stand on the clients pane and nest under nothing.
mod clients;
/// The config pane's two, one of which addresses a channel rather than a wall.
mod config;
/// The provider table, which stands on the login pane and nests under nothing.
mod login;
/// The three questions, and how each waits for the last to have an answer.
mod nesting;
/// The union across channels, and the channel that costs only itself.
mod union;
