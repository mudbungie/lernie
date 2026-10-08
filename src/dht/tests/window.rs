//! The sliding window (yog bl-d9c1): a silent query costs its own slot for one
//! deadline and never delays an answer beside it, and a slot is refilled the
//! moment its query answers or times out.
//!
//! Time is the [`steered`] clock's, which moves only past silence: a wait is
//! measured exactly, and never against how loaded the box was (bl-7b83).

use super::steered::steered;
use super::*;

/// The door names a live node and a silent one together, and the live one
/// names a closer live one. Lockstep, the walk waited out the silent query's
/// whole round before asking the closer node; windowed, the closer node is
/// asked the moment the live one answers, and once the two live nodes are
/// the K closest the walk ends without waiting on the silent one at all.
#[test]
fn a_silent_node_does_not_delay_an_answer_in_the_same_window() {
    let mut near = FakeNode::bind(id(0xf0));
    near.serve(vec![], Mood::Answer, vec![]);
    let mut live = FakeNode::bind(id(0x80));
    live.serve(vec![near.node()], Mood::Answer, vec![]);
    let mut silent = FakeNode::bind(id(0x10));
    silent.serve(vec![], Mood::Silent, vec![]);
    let door = router(vec![silent.node(), live.node()]);
    let config = Config {
        k: 2,
        deadline: Duration::from_secs(2),
        ..quick()
    };
    let (mut dht, clock) = steered(vec![door.addr], config, &[&silent]);
    let started = clock.arc().now();
    assert_eq!(
        lookup(&mut dht, id(0xff)).unwrap(),
        vec![near.node(), live.node()]
    );
    assert_eq!(clock.arc().now(), started, "no silence was waited on");
}

/// One slot, the closest node silent: its deadline passes, the slot is
/// refilled with the next node on the frontier, and that one answers.
#[test]
fn the_window_refills_when_a_query_times_out() {
    let mut silent = FakeNode::bind(id(0xf0));
    silent.serve(vec![], Mood::Silent, vec![]);
    let mut live = FakeNode::bind(id(0x10));
    live.serve(vec![], Mood::Answer, vec![]);
    let door = router(vec![silent.node(), live.node()]);
    let config = Config {
        alpha: 1,
        deadline: Duration::from_millis(200),
        ..quick()
    };
    let (mut dht, clock) = steered(vec![door.addr], config, &[&silent]);
    let started = clock.arc().now();
    assert_eq!(lookup(&mut dht, id(0xff)).unwrap(), vec![live.node()]);
    assert_eq!(clock.arc().now() - started, Duration::from_millis(200));
}

/// A holder that offered a token and is silent to `put` costs its own
/// deadline, and the holders beside it are still counted.
#[test]
fn a_put_counts_the_holders_that_answer_past_a_silent_one() {
    let kp = Keypair::from_seed([3u8; 32]).unwrap();
    let item = kp.sign(vec![], 1, b"p".to_vec()).unwrap();
    let mut holder = FakeNode::bind(id(0x42));
    holder.serve(vec![], Mood::Answer, vec![]);
    let mut mute = FakeNode::bind(id(0x43));
    mute.serve(vec![], Mood::Mute, vec![]);
    let door = router(vec![holder.node(), mute.node()]);
    let (mut dht, _) = steered(vec![door.addr], quick(), &[&mute]);
    assert_eq!(dht.put(item).unwrap(), 1);
}
