//! **The queue lane** (bl-5439): the one standing read that is slow enough to
//! need a thread of its own.
//!
//! `attention` is asked of every channel while the queue's pane is open, and
//! while the selected conversation is asking, because the ask above the
//! prompt is the queue's row for it (DESIGN §4.42, item 8). A server can take
//! half a minute to answer it. In the asker's serial pass that half minute was
//! paid by every read queued behind it — the trail, the board, the records
//! and the selected conversation's transcript all said *waiting to hear* for
//! as long — so it is asked here, on its own socket and its own cadence, and
//! the pass that paints everything else never waits for it.

use std::path::Path;

use crate::state::{Link, Open};

/// **Ask every channel what is waiting**, if anything on the glass stands on
/// the answer; otherwise ask nothing.
pub fn tick(link: &Link, root: &Path) {
    let standing = link.standing();
    if !(standing.standing(&Open::Queue) || standing.asking) {
        return;
    }
    for channel in &standing.channels {
        super::asker::read(
            link,
            super::down(link, root, channel, &crate::verbs::attention()),
            channel,
        );
    }
}
