//! **The records pane's reads** (§4.29, bl-2cf7): what the selected
//! conversation's loop did, what its worktree holds, which config governs it
//! and what sits in its inbox — asked only while somebody is looking. Split
//! from [`super::workspace`] at the design-time budget (bl-1d79), when the
//! spine left this fan for the selection's own (§4.47 item 3).

use std::path::Path;

use super::aimed;
use super::workspace::about;
use crate::state::Link;
use crate::ui::Channel;

/// **The records pane's six standing reads**, asked only while it is open.
///
/// A body of its own rather than six more calls inside [`ask`]: that function
/// is *the questions that name one workspace*, and this is one pane's — the
/// pane with more reads under it than any other, and the one three balls grew.
/// The pair bl-2cf7 landed is what the loop did and what its worktree holds;
/// bl-b52c's spine is the two whose answer the `fork` control's one argument
/// comes off, because a control offered on a notch this seat has not been
/// answered about would be a control with nothing to carry; and bl-3257's two
/// were the conversation's own row and its undelivered mail, of which the row
/// went back out to the conversation's own standing set at bl-7b03 — this pane
/// still reads it, off the same field, but the header over the TRANSCRIPT
/// needs it whether or not this pane is open. The sixth is the workspace's lineage
/// listing (bl-ed20), the config pane's read asked here too, because the names
/// it answers are the only arguments the workflow mark's control can carry
/// (`crate::ui::records::workflow`) — the spine's reason, one read over. It
/// lands on the one `Model::lineages` either pane fills.
///
/// **The pane's seventh read is not here and never will be**: `step` is
/// addressed at one row of the ledger rather than at the pane, so the control
/// on that row posts it (`crate::ui::model::deep`).
pub(super) fn records(
    link: &Link,
    root: &Path,
    channel: &Channel,
    workspace: &str,
    conversation: &str,
) {
    for ask in [
        crate::verbs::steps(workspace.to_owned(), conversation.to_owned()),
        crate::verbs::files(workspace.to_owned(), conversation.to_owned()),
        crate::verbs::governing(workspace.to_owned(), conversation.to_owned()),
        crate::verbs::inbox(workspace.to_owned(), conversation.to_owned()),
    ] {
        about(link, root, channel, conversation, &ask);
    }
    aimed(
        link,
        root,
        channel,
        &crate::verbs::lineages(workspace.to_owned()),
    );
}
