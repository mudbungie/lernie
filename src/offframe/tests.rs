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

use super::{follow, pump, run};
use crate::state::Link;
use crate::test_support::Scratch;
use crate::test_support::server::Answer;
use crate::test_support::wire::{flat, wired};
use crate::ui::{Aim, Channel, Chunk, Model};
use serde_json::json;

/// The beats that drive the whole seat and read the glass.
mod window;

/// How long a test waits on the threads before it calls them HUNG — never how
/// long a slow box gets (bl-7b83). Twenty seconds was a bet, and the builder
/// under coverage, other suites beside it, lost it before the first dial
/// answered. Every wait on it ends on the state it watches, so a healthy box
/// spends milliseconds; it sits under tarpaulin's own 300 s so the failure
/// that names the wait is this suite's and not the runner's.
const HUNG: Duration = Duration::from_mins(4);

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
            dials: None,
        })],
        ..Model::default()
    };
    let link = Link::new(Duration::from_millis(1));
    link.settle(&mut model);
    let workers = run(&link, scratch.path());
    // Settle until the answer lands or the deadline says the threads are not
    // working. A bare sleep would be a race written down; a deadline is the
    // assertion that this finishes at all.
    let deadline = std::time::Instant::now() + HUNG;
    while model.roster[0].workspaces.is_empty() && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
        link.settle(&mut model);
    }
    link.stop();
    for worker in workers {
        worker.join().expect("a worker");
    }
    assert_eq!(
        model.roster[0]
            .workspaces
            .first()
            .map(|row| row.workspace.clone()),
        Some("home".to_owned()),
        "the asker's answer reached the frame"
    );
}

/// **A stop does not end a held lane, so the exit must not wait on one**
/// (bl-9cef). The follow lane is parked on a socket read and learns of a stop
/// only when the server writes or hangs up — its ping, twenty-five seconds on a
/// real server — which is the hang a join at exit became. So `src/main.rs`
/// drops the handle, and this is the fact that makes dropping it the only
/// answer: the lane is still running after the stop, and nothing but the
/// server or the process's end will finish it.
#[test]
fn a_stop_leaves_a_held_lane_parked_so_the_exit_joins_nothing() {
    let scratch = Scratch::new();
    wired(
        &scratch,
        &flat(),
        vec![Answer::Held(vec![
            json!({"ok": true, "kind": "follow", "tools": [],
                   "stream": {"text": "a tail", "delta": "text"}}),
        ])],
    );
    let own = Channel {
        name: crate::seat::OWN.to_owned(),
        named_there: None,
        dials: None,
    };
    let mut model = Model {
        aim: Some(Aim {
            channel: own.name.clone(),
            address: "home".to_owned(),
        }),
        conversation: Some("20260830T051200Z-a1b2".to_owned()),
        roster: vec![Chunk::of(own)],
        ..Model::default()
    };
    let link = Link::new(Duration::from_millis(1));
    link.settle(&mut model);
    let lane = {
        let link = link.clone();
        let root = scratch.path().to_path_buf();
        std::thread::spawn(move || pump(&link, || follow::tick(&link, &root)))
    };
    // The first frame landing is the proof the lane is on the line; from here
    // it is parked on the next read, which the stand-in holds.
    let deadline = std::time::Instant::now() + HUNG;
    while model.live.is_none() && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
        link.settle(&mut model);
    }
    assert!(
        model.live.is_some(),
        "the held read's first frame reached the frame"
    );
    link.stop();
    assert!(
        !lane.is_finished(),
        "the stop did not reach the held read: a join here would wait out the server"
    );
    drop(lane);
}
