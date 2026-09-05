//! **The asker**: one pass over the standing question set.
//!
//! The questions nest. Every channel is asked for its own roster — and, while
//! the decision queue is open, for what is asking on it (bl-f0ef), and, while
//! the trail is open, for what has crossed its boundary (bl-4c48) — which is
//! what makes all three a **union across channels**, composed here
//! rather than anywhere on the wire. The aimed wall is asked for its
//! conversations, and — while the tuning pane is open on it — for what its
//! roles are set to, and — while the login pane is open on it — for what it
//! can sign in to (bl-e3c5). The selected conversation is asked for its
//! transcript,
//! and — while the records pane is open on it — for its steps, its worktree's
//! files (bl-2cf7), its spine and the config commit governing it (bl-b52c).
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

use super::down;
use crate::state::{Link, Open, Said};
use crate::ui::Channel;

/// Ask everything the last frame said to ask — the questions about every
/// channel, then the questions about the focus.
///
/// **Split in two at the design-time budget on the seam the module's own doc
/// draws** (bl-5c53): [`everywhere`] is the reads whose subject is *every
/// channel this box holds*, and [`focused`] is the nest under the aim and the
/// selection. One grows when a channel-wide op lands a pane; the other when a
/// pane about a focus does.
pub fn tick(link: &Link, root: &Path) {
    let standing = link.standing();
    everywhere(link, root, &standing);
    focused(link, root, &standing);
}

/// **The reads whose subject is every channel**: each channel's own roster,
/// and the two pane-keyed unions composed across them.
fn everywhere(link: &Link, root: &Path, standing: &crate::state::Standing) {
    for channel in &standing.channels {
        read(
            link,
            down(link, root, channel, &crate::verbs::workspaces()),
            channel,
        );
        // **The queue fans with the roster** (bl-f0ef), and for the same
        // reason: `attention` names no workspace, so its subject is every
        // channel this box holds and the union is composed here. It stands on
        // the PANE rather than on a focus, because nothing on the glass is its
        // subject — see `crate::state::Standing::queue`.
        if standing.standing(&Open::Queue) {
            read(
                link,
                down(link, root, channel, &crate::verbs::attention()),
                channel,
            );
        }
        // **And the trail fans on the same terms** (bl-4c48): `ops` names no
        // workspace either, so its subject is every channel and the pane is
        // the union. It STANDS rather than being posted once, unlike the
        // window's other two channel-wide reads, because a trail is what is
        // happening: every act this seat spends appends a row to it, and an
        // alarm goes up and comes down under an operator who is looking.
        if standing.standing(&Open::Trail) {
            read(
                link,
                down(link, root, channel, &crate::verbs::ops(crate::verbs::DEPTH)),
                channel,
            );
        }
    }
}

/// **The nest under the focus**: the aimed wall's questions, the panes keyed
/// on it, and the selected conversation's.
fn focused(link: &Link, root: &Path, standing: &crate::state::Standing) {
    let Some((channel, aim)) = standing.aimed() else {
        return;
    };
    aimed(
        link,
        root,
        &channel,
        &crate::verbs::conversations(aim.address.clone()),
    );
    if standing.standing(&Open::Tuning) {
        aimed(
            link,
            root,
            &channel,
            &crate::verbs::roles(aim.address.clone()),
        );
    }
    // **The provider table stands on the login pane** (bl-e3c5), on the roles
    // read's own terms — and the standing is what makes it worth having: a
    // credential lands on the engine while the operator is looking at the
    // table, so a row that said *no credential* says otherwise on the next
    // beat with nothing asked again. The run's own lines are the held lane's
    // (`crate::offframe::signin`), never this pass's.
    if standing.standing(&Open::Login(standing.signin())) {
        aimed(
            link,
            root,
            &channel,
            &crate::verbs::providers(aim.address.clone()),
        );
    }
    // **The machines stand on their pane** (bl-e53c), on the provider table's
    // own terms and for a sharper form of its reason: a row's presence is true
    // only at the instant the engine answered it, so the read that says a foot
    // is connected is worth nothing unless it is asked again.
    if standing.standing(&Open::Clients) {
        aimed(
            link,
            root,
            &channel,
            &crate::verbs::clients(aim.address.clone()),
        );
    }
    // **The config pane's two, and the one whose subject is a CHANNEL**
    // (bl-5c53). The lineage listing carries the aim's workspace and is routed
    // by it like every other aimed read. The file read is routed by it only
    // for the two destinations that name one: litany's globals and yog's own
    // cadence file belong to the ENGINE, so what they address is the channel
    // the window is aimed at — asked here the way a roster is, by name, rather
    // than falling through to this box's own engine (DESIGN §4.30).
    if standing.standing(&Open::Config(standing.at())) {
        aimed(
            link,
            root,
            &channel,
            &crate::verbs::lineages(aim.address.clone()),
        );
        if let Some(at) = standing.at() {
            let gesture = crate::verbs::config(&at);
            if at.addresses_a_workspace() {
                aimed(link, root, &channel, &gesture);
            } else {
                read(link, down(link, root, &channel, &gesture), &channel);
            }
        }
    }
    let Some(conversation) = standing.conversation.clone() else {
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
    if standing.standing(&Open::Records) {
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
            &crate::verbs::files(aim.address.clone(), conversation.clone()),
        );
        // **And the spine pair with them** (bl-b52c): the same pane, and the
        // reads whose answer the `fork` control's one argument comes off — a
        // control offered on a notch this seat has not been answered about
        // would be a control with nothing to carry.
        aimed(
            link,
            root,
            &channel,
            &crate::verbs::rail(aim.address.clone(), conversation.clone()),
        );
        aimed(
            link,
            root,
            &channel,
            &crate::verbs::governing(aim.address, conversation),
        );
    }
}

/// **A question addressed at a WORKSPACE**, which is the ordinary path: the
/// envelope carries the address the roster handed out, and
/// [`route`](crate::seat::route) resolves it over this box's entries and
/// rewrites it to the host's spelling at the one place that mapping is spent.
fn aimed(link: &Link, root: &Path, channel: &Channel, envelope: &Value) {
    let asked = crate::seat::route(root, envelope)
        .sent
        .map_err(crate::channel::Reach::Unsent)
        .and_then(|(open, carried)| open.ask(&carried));
    match asked {
        Ok(stream) => super::file(link, channel, stream),
        Err(reach) => read(link, Err(reach), channel),
    }
}

/// **A read's failure is the channel's own relationship**, whichever leg
/// produced it, and it says nothing about whether the request crossed:
/// re-asking is free (REMOTE §3: *"a read is answered in place, and asking
/// twice is asking once"*), so a standing question needs no arm for a fact it
/// would do nothing with. The whole set is asked again on the next beat.
fn read(link: &Link, leg: Result<(), crate::channel::Reach>, channel: &Channel) {
    if let Err(reach) = leg {
        link.heard(channel, Said::Unreachable(reach.said()));
    }
}

#[cfg(test)]
mod tests;
