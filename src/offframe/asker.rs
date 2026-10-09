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
//! files (bl-2cf7), its spine and the config commit governing it (bl-b52c),
//! and the wall's lineages a workflow mark can name (bl-ed20).
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

/// Ask everything the last frame said to ask — the selected conversation's
/// reads first, then the questions about every channel, then the rest of the
/// focus.
///
/// **Split in two at the design-time budget on the seam the module's own doc
/// draws** (bl-5c53): [`fanned`], once per channel, is the reads whose subject
/// is *every channel this box holds*, and [`focused`] is the nest under the aim. One
/// grows when a channel-wide op lands a pane; the other when a pane about a
/// focus does.
///
/// **That seam is the wire's own** ([`crate::verbs::Verb::addresses_a_workspace`]):
/// a question with no workspace field has no way to name a channel, so it goes
/// down every one and the pane is the union; a question with one goes down the
/// aimed wall's alone. A pane that asks at BOTH widths therefore appears in
/// both halves — which is a fact about its four ops rather than a special case
/// (`crate::ui::board`, DESIGN §4.31).
///
/// **A selection is answered at the next leg, not at the next pass** (bl-1f22).
/// Selecting a conversation empties its transcript, and over a dialed wire one
/// pass of roster fans is seconds — so a selection that waited its turn showed
/// an empty pane for all of them. Before every leg the pass re-reads the
/// standing set, and a selection it has not asked about yet is asked about
/// then, down the one channel it names. The pass's own first leg is the same
/// rule with nothing asked so far, which is what puts the selection's reads
/// ahead of the fan; no leg is skipped for it, so the sweep still visits every
/// channel.
pub fn tick(link: &Link, root: &Path) {
    let standing = link.standing();
    let mut asked = None;
    for channel in &standing.channels {
        asked = caught_up(link, root, asked);
        fanned(link, root, &standing, channel);
    }
    caught_up(link, root, asked);
    focused(link, root, &standing);
}

/// **The selection this pass has asked about**: the channel and wall it is on,
/// and its id. A pair rather than the id alone, because a selection moves when
/// either half does.
type Asked = Option<(Channel, crate::ui::Aim, String)>;

/// **Ask the selection's reads if the selection has moved since `asked`**,
/// and answer what is now asked about.
///
/// The standing set is read fresh here rather than taken from the pass's start:
/// it is the whole of how a click made mid-pass is seen before the pass ends.
fn caught_up(link: &Link, root: &Path, asked: Asked) -> Asked {
    let now = link.standing();
    let selection = now
        .aimed()
        .zip(now.conversation.clone())
        .map(|((channel, aim), conversation)| (channel, aim, conversation));
    if selection != asked
        && let Some((channel, aim, conversation)) = &selection
    {
        wall::selected(link, root, &now, channel, &aim.address, conversation);
    }
    selection
}

/// **The nest under the aim**: the aimed wall's questions and the panes keyed
/// on it — [`wall`]'s. The selected conversation's are [`caught_up`]'s.
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
