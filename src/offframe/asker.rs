//! **The asker**: one pass over the standing question set.
//!
//! Three questions and they nest. Every channel is asked for its own roster —
//! which is what makes the roster a **union across channels**, composed here
//! rather than anywhere on the wire. The aimed wall is asked for its
//! conversations. The selected conversation is asked for its transcript.
//!
//! **A channel that will not answer costs only itself.** Each leg reports its
//! own outcome, so a box holding three channels with one engine down still gets
//! the two that are up — REMOTE §8.2's *"a refusal is one entry's, never the
//! set's"*, one layer above the file it was written about.

use std::path::Path;

use serde_json::Value;

use crate::state::{Link, Said};
use crate::ui::Channel;

/// Ask everything the last frame said to ask.
pub fn tick(link: &Link, root: &Path) {
    let standing = link.standing();
    for channel in &standing.channels {
        down(link, root, channel, &crate::verbs::workspaces());
    }
    let Some((channel, aim)) = standing.aimed() else {
        return;
    };
    aimed(
        link,
        root,
        &channel,
        &crate::verbs::conversations(aim.address.clone()),
    );
    let Some(conversation) = standing.conversation else {
        return;
    };
    aimed(
        link,
        root,
        &channel,
        &crate::verbs::transcript(aim.address, conversation),
    );
}

/// **A question addressed at a CHANNEL**, not at a workspace: the roster read
/// names none, so there is nothing for the §8.2 mapping to resolve and the
/// channel has to be opened by the name this box knows it by.
fn down(link: &Link, root: &Path, channel: &Channel, envelope: &Value) {
    match crate::seat::dial(root, &channel.name).and_then(|open| open.ask(envelope)) {
        Ok(stream) => file(link, channel, stream),
        Err(why) => link.heard(channel, Said::Unreachable(why)),
    }
}

/// **A question addressed at a WORKSPACE**, which is the ordinary path: the
/// envelope carries the address the roster handed out, and
/// [`route`](crate::seat::route) resolves it over this box's entries and
/// rewrites it to the host's spelling at the one place that mapping is spent.
fn aimed(link: &Link, root: &Path, channel: &Channel, envelope: &Value) {
    match crate::seat::route(root, envelope).and_then(|(open, carried)| open.ask(&carried)) {
        Ok(stream) => file(link, channel, stream),
        Err(why) => link.heard(channel, Said::Unreachable(why)),
    }
}

/// Every frame of one answer, in order. **An answer of no frames is reported as
/// nothing**, which is what it is: the engine terminated the stream without
/// saying anything, and a seat that invented a sentence for it would be
/// speaking for an engine that did not.
fn file(link: &Link, channel: &Channel, stream: Vec<Value>) {
    for frame in stream {
        link.heard(channel, Said::Frame(frame));
    }
}

#[cfg(test)]
mod tests;
