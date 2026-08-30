//! **The whole seat, driven**: what an operator's gesture does when a real
//! listener is behind the channel and the window is what reads the answer.
//!
//! Split from [`super`] at the design-time budget on the seam the two halves
//! have — that suite is about the threads, and this is about what the seat
//! *does*, every assertion ending on the glyphs that reached the glass.

use std::time::Duration;

use super::super::{poster, run};
use crate::state::Link;
use crate::test_support::Scratch;
use crate::test_support::wire::{flat, wired};
use crate::ui::{Channel, Chunk, Model};
use serde_json::json;

/// **The whole seat, in one process**: a real listener with a real mTLS
/// handshake and a real version preface, the three threads, the settle, and the
/// window — asserted on the glyphs that reached the glass.
///
/// It is the beat the ball is ultimately about, and it is the only one that
/// crosses every seam at once: everything eframe adds beyond this is an event
/// loop and a GL surface, which is why the entry point that holds them decides
/// nothing.
#[test]
fn the_window_paints_what_a_real_engine_answered() {
    let scratch = Scratch::new();
    let roster = json!({"ok": true, "kind": "workspaces",
                        "rows": [{"workspace": "home", "kind": "named", "attention": 2,
                                  "agents": 3, "running": true}]});
    wired(&scratch, &flat(), vec![vec![roster]; 8]);
    let mut model = Model {
        roster: crate::seat::channels(scratch.path())
            .into_iter()
            .map(Chunk::of)
            .collect(),
        ..Model::default()
    };
    let link = Link::new(Duration::from_millis(1));
    link.settle(&mut model);
    let workers = run(&link, scratch.path());
    let window = crate::paint_probe::frame::Window::new();
    let deadline = std::time::Instant::now() + Duration::from_secs(20);
    let mut painted = String::new();
    while !painted.contains("home  (named)  3 conversations  2 waiting  running")
        && std::time::Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(5));
        link.settle(&mut model);
        painted = window.text(|ctx| crate::ui::render(ctx, &mut model));
    }
    link.stop();
    for worker in workers {
        worker.join().expect("a worker");
    }
    assert!(
        painted.contains("home  (named)  3 conversations  2 waiting  running"),
        "the engine's own answer never reached the glass:\n{painted}"
    );
}

/// **The seat begins a conversation, end to end.** The window composes the
/// staging act, the poster sends it against a real listener, the frame that
/// absorbs its answer composes the fire, the poster sends *that*, and the
/// minted name reaches the glass.
///
/// It is the one beat that proves the two-act shape rather than either act: the
/// second envelope exists only because the first was answered, and the thing
/// between them — the staged body — was held by the model across a round trip.
#[test]
fn the_window_begins_a_conversation_in_two_acts_and_paints_the_minted_name() {
    let scratch = Scratch::new();
    let engine = wired(
        &scratch,
        &flat(),
        vec![
            vec![json!({"ok": true, "kind": "prepared",
                        "prepared": {"workspace": "home", "goal": "", "origin": "world"}})],
            vec![json!({"ok": true, "kind": "started", "conversation": "brisk-otter"})],
        ],
    );
    let link = Link::new(Duration::from_millis(1));
    let mut model = Model {
        roster: vec![Chunk::of(Channel {
            name: crate::seat::OWN.to_owned(),
            named_there: None,
        })],
        aim: Some(crate::ui::Aim {
            channel: crate::seat::OWN.to_owned(),
            address: "home".to_owned(),
        }),
        draft: "do the thing".to_owned(),
        ..Model::default()
    };
    model.stage("home");
    let window = crate::paint_probe::frame::Window::new();
    let mut painted = String::new();
    // Two passes, because the flow is two acts: the second is composed by the
    // frame that settles the first's answer, so it cannot exist before it.
    for _ in 0..2 {
        link.settle(&mut model);
        poster::tick(&link, scratch.path());
        link.settle(&mut model);
        painted = window.text(|ctx| crate::ui::render(ctx, &mut model));
    }
    let heard = engine.heard();
    assert!(
        heard
            .iter()
            .any(|said| said["op"] == json!("prepare") && said["workspace"] == json!("home")),
        "{heard:?}"
    );
    assert!(
        heard
            .iter()
            .any(|said| said["op"] == json!("prompt") && said["goal"] == json!("do the thing")),
        "{heard:?}"
    );
    assert!(
        painted.contains("started «brisk-otter» in home"),
        "the minted name never reached the glass:\n{painted}"
    );
}
