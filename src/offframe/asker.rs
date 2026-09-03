//! **The asker**: one pass over the standing question set.
//!
//! The questions nest. Every channel is asked for its own roster — and, while
//! the decision queue is open, for what is asking on it (bl-f0ef) — which is
//! what makes both a **union across channels**, composed here
//! rather than anywhere on the wire. The aimed wall is asked for its
//! conversations, and — while the tuning pane is open on it — for what its
//! roles are set to. The selected conversation is asked for its transcript,
//! and — while the records pane is open on it — for its steps and its
//! worktree's files (bl-2cf7).
//!
//! **The roles read is standing rather than one-shot**, which is what lets
//! every control on that pane state the engine's fact instead of this end's
//! prediction: a tuning act is composed, sent, and read back on the next beat.
//! A pane that wrote its own row would be holding a second opinion about a file
//! it does not own, and the three writes go through a `litany config` that can
//! refuse.
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
        // **The queue fans with the roster** (bl-f0ef), and for the same
        // reason: `attention` names no workspace, so its subject is every
        // channel this box holds and the union is composed here. It stands on
        // the PANE rather than on a focus, because nothing on the glass is its
        // subject — see `crate::state::Standing::queue`.
        if standing.queue {
            down(link, root, channel, &crate::verbs::attention());
        }
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
    if standing.tuning {
        aimed(
            link,
            root,
            &channel,
            &crate::verbs::roles(aim.address.clone()),
        );
    }
    let Some(conversation) = standing.conversation else {
        return;
    };
    aimed(
        link,
        root,
        &channel,
        &crate::verbs::transcript(aim.address.clone(), conversation.clone()),
    );
    // **The records pair stands on the pane exactly as the roles read does**
    // (bl-2cf7): the selected conversation is asked what its loop did and
    // what its worktree holds only while somebody is looking.
    if standing.records {
        aimed(
            link,
            root,
            &channel,
            &crate::verbs::steps(aim.address.clone(), conversation.clone()),
        );
        aimed(
            link,
            root,
            &channel,
            &crate::verbs::files(aim.address, conversation),
        );
    }
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
