//! The threading itself: the loop that stops, and one end-to-end beat where
//! real threads carry a real answer to a real frame.
//!
//! Split at the design-time budget on the seam the two halves have: this is
//! about the *threads* — that they start, pass and stop — and [`window`] is
//! about what the whole seat does with what they bring, read off the glass.
//!
//! Split at the design-time budget on the seam the two halves have: this is
//! about the *threads* — that they start, pass and stop — and [`window`] is
//! about what the whole seat does with what they bring, read off the glass.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use super::{pump, run};
use crate::state::Link;
use crate::test_support::Scratch;
use crate::test_support::wire::{flat, wired};
use crate::ui::{Channel, Chunk, Model};
use serde_json::json;

/// The beats that drive the whole seat and read the glass.
mod window;

/// **A pass runs, then the loop asks whether to run again.** So a stop is seen
/// between passes and never during one — which is what lets every worker's body
/// be a plain function with nothing to check halfway through.
#[test]
fn a_worker_passes_until_the_link_says_stop() {
    let link = Link::new(Duration::from_millis(1));
    let passes = AtomicUsize::new(0);
    pump(&link, || {
        if passes.fetch_add(1, Ordering::Relaxed) >= 2 {
            link.stop();
        }
    });
    assert_eq!(
        passes.load(Ordering::Relaxed),
        3,
        "three passes, then the check"
    );
}

/// A link that was already stopped runs no pass at all — the check is at the
/// top, so a worker started after a shutdown does nothing rather than one
/// thing.
#[test]
fn a_worker_started_after_a_stop_does_nothing() {
    let link = Link::new(Duration::from_millis(1));
    link.stop();
    let passes = AtomicUsize::new(0);
    pump(&link, || {
        passes.fetch_add(1, Ordering::Relaxed);
    });
    assert_eq!(passes.load(Ordering::Relaxed), 0);
}

/// **The one end-to-end beat**: three real threads, a real listener, and a
/// frame that settles what they brought.
///
/// Everything about what a pass *does* is asserted beside that pass; what this
/// one is about is the threading — that `run` starts them, that a stop and a
/// join end them, and that what they heard reaches the window through the one
/// door the frame uses.
#[test]
fn the_threads_carry_a_real_answer_to_a_real_frame() {
    let scratch = Scratch::new();
    // Enough connections that whichever worker dials first is served: the three
    // run at once and the roster read is the only one the standing set implies.
    let roster = json!({"ok": true, "kind": "workspaces",
                        "rows": [{"workspace": "home", "kind": "named", "attention": 0,
                                  "agents": 3, "running": true}]});
    wired(&scratch, &flat(), vec![vec![roster.clone()]; 8]);
    let mut model = Model {
        roster: vec![Chunk::of(Channel {
            name: crate::seat::OWN.to_owned(),
            named_there: None,
        })],
        ..Model::default()
    };
    let link = Link::new(Duration::from_millis(1));
    link.settle(&mut model);
    let workers = run(&link, scratch.path());
    // Settle until the answer lands or the deadline says the threads are not
    // working. A bare sleep would be a race written down; a deadline is the
    // assertion that this finishes at all.
    let deadline = std::time::Instant::now() + Duration::from_secs(20);
    while model.roster[0].walls.is_empty() && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
        link.settle(&mut model);
    }
    link.stop();
    for worker in workers {
        worker.join().expect("a worker");
    }
    assert_eq!(
        model.roster[0]
            .walls
            .first()
            .map(|row| row.workspace.clone()),
        Some("home".to_owned()),
        "the asker's answer reached the frame"
    );
}
