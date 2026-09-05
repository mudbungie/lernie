//! **The questions that name one workspace**, asked of the aimed wall.
//!
//! Split from [`super`] at the design-time budget on the seam that module's
//! own doc already draws (`crate::verbs::Verb::addresses_a_workspace`): a
//! question with no workspace field goes down every channel this box holds and
//! its pane is the union, and a question with one goes down the aimed wall's
//! alone. This is the second, and it is where six panes' reads nest — which is
//! why it is the half that grows.
//!
//! **A pane that asks at BOTH widths appears here AND in [`super::fanned`]**,
//! and that is a fact about its ops rather than a special case: the ball pane
//! reads `balls` and `board` of every channel and `workspace-balls` and
//! `marks` of the aimed wall, from one standing (DESIGN §4.31).

use std::path::Path;

use super::{aimed, down, read};
use crate::state::{Link, Open};
use crate::ui::Channel;

/// **The questions that name one workspace**, asked of the aimed wall.
pub(super) fn ask(
    link: &Link,
    root: &Path,
    standing: &crate::state::Standing,
    channel: &Channel,
    aim: &crate::ui::Aim,
) {
    aimed(
        link,
        root,
        channel,
        &crate::verbs::conversations(aim.address.clone()),
    );
    if standing.standing(&Open::Tuning) {
        aimed(
            link,
            root,
            channel,
            &crate::verbs::roles(aim.address.clone()),
        );
    }
    // **And the ball pane's other two are AIMED** (bl-d2af), on the roles
    // read's own terms: the wall the operator is looking at is asked what
    // balls it holds and which branch it tracks them on, while the same pane
    // stands. One pane, two widths, because that is what its four ops are.
    if standing.standing(&Open::Board) {
        aimed(
            link,
            root,
            channel,
            &crate::verbs::workspace_balls(aim.address.clone()),
        );
        aimed(
            link,
            root,
            channel,
            &crate::verbs::marks(aim.address.clone()),
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
            channel,
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
            channel,
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
            channel,
            &crate::verbs::lineages(aim.address.clone()),
        );
        if let Some(at) = standing.at() {
            let gesture = crate::verbs::config(&at);
            if at.addresses_a_workspace() {
                aimed(link, root, channel, &gesture);
            } else {
                read(link, down(link, root, channel, &gesture), channel);
            }
        }
    }
    let Some(conversation) = standing.conversation.clone() else {
        return;
    };
    aimed(
        link,
        root,
        channel,
        &crate::verbs::transcript(aim.address.clone(), conversation.clone()),
    );
    // **The records pane's reads stand on the pane exactly as the roles read
    // does** (bl-2cf7): the selected conversation is asked what its loop did
    // and what its worktree holds only while somebody is looking.
    if standing.standing(&Open::Records) {
        records(link, root, channel, &aim.address, &conversation);
    }
}

/// **The records pane's six standing reads**, asked only while it is open.
///
/// A body of its own rather than six more calls inside [`ask`]: that function
/// is *the questions that name one workspace*, and this is one pane's — the
/// pane with more reads under it than any other, and the one three balls grew.
/// The pair bl-2cf7 landed is what the loop did and what its worktree holds;
/// bl-b52c's spine is the two whose answer the `fork` control's one argument
/// comes off, because a control offered on a notch this seat has not been
/// answered about would be a control with nothing to carry; and bl-3257's two
/// are the conversation's own row and its undelivered mail.
///
/// **The pane's seventh read is not here and never will be**: `step` is
/// addressed at one row of the ledger rather than at the pane, so the control
/// on that row posts it (`crate::ui::model::deep`).
fn records(link: &Link, root: &Path, channel: &Channel, wall: &str, conversation: &str) {
    for ask in [
        crate::verbs::steps(wall.to_owned(), conversation.to_owned()),
        crate::verbs::files(wall.to_owned(), conversation.to_owned()),
        crate::verbs::rail(wall.to_owned(), conversation.to_owned()),
        crate::verbs::governing(wall.to_owned(), conversation.to_owned()),
        crate::verbs::agent(wall.to_owned(), conversation.to_owned()),
        crate::verbs::inbox(wall.to_owned(), conversation.to_owned()),
    ] {
        aimed(link, root, channel, &ask);
    }
}
