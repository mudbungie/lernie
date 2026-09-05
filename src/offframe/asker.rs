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

/// The questions that name one workspace, asked of the aimed wall.
mod wall;
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
///
/// **That seam is the wire's own** ([`crate::verbs::Verb::addresses_a_workspace`]):
/// a question with no workspace field has no way to name a channel, so it goes
/// down every one and the pane is the union; a question with one goes down the
/// aimed wall's alone. A pane that asks at BOTH widths therefore appears in
/// both halves — which is a fact about its four ops rather than a special case
/// (`crate::ui::board`, DESIGN §4.31).
pub fn tick(link: &Link, root: &Path) {
    let standing = link.standing();
    everywhere(link, root, &standing);
    focused(link, root, &standing);
}

/// **The reads whose subject is every channel**: each channel's own roster,
/// and the two pane-keyed unions composed across them.
fn everywhere(link: &Link, root: &Path, standing: &crate::state::Standing) {
    for channel in &standing.channels {
        fanned(link, root, standing, channel);
    }
}

/// **The nest under the focus**: the aimed wall's questions, the panes keyed
/// on it, and the selected conversation's — all of it [`wall`]'s.
fn focused(link: &Link, root: &Path, standing: &crate::state::Standing) {
    let Some((channel, aim)) = standing.aimed() else {
        return;
    };
    wall::ask(link, root, standing, &channel, &aim);
}

/// **The questions that name no workspace**, asked of one channel — the caller
/// asks them of every channel this box holds, which is what makes each pane
/// above the union.
fn fanned(link: &Link, root: &Path, standing: &crate::state::Standing, channel: &Channel) {
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
    // **The ball pane's two widest reads fan on the same terms** (bl-d2af):
    // `balls` is the whole box's binding table and `board` its fold into
    // columns, and neither names a workspace. Both stand while the pane is
    // open, for the trail's reason — a board is what is happening, and a
    // claim, a spawn or a loop's tick moves it under an operator looking
    // at it.
    if standing.standing(&Open::Board) {
        read(
            link,
            down(link, root, channel, &crate::verbs::balls()),
            channel,
        );
        read(
            link,
            down(link, root, channel, &crate::verbs::board()),
            channel,
        );
    }
}

/// **A question addressed at a WORKSPACE**, which is the ordinary path: the
/// envelope carries the address the roster handed out, and
/// [`route`](crate::seat::route) resolves it over this box's entries and
/// rewrites it to the host's spelling at the one place that mapping is spent.
pub(super) fn aimed(link: &Link, root: &Path, channel: &Channel, envelope: &Value) {
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
pub(super) fn read(link: &Link, leg: Result<(), crate::channel::Reach>, channel: &Channel) {
    if let Err(reach) = leg {
        link.heard(channel, Said::Unreachable(reach.said()));
    }
}

#[cfg(test)]
mod tests;
