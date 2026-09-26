//! A line between asks: the silence bound on the injected clock, the ping
//! discard, every way the engine can end it, and the follower that stops
//! early.

use super::*;

#[test]
fn a_held_line_is_judged_gone_one_ping_before_the_bound() {
    let (listener, at) = listener();
    let b = Bench::new(dead(), Some(presence(at)));
    let engine = Engine::listen(
        b.scratch.path(),
        listener,
        vec![Reply::yes(), Reply::yes(), Reply::yes()],
    );
    let channel = b.channel();
    assert!(channel.ask(&request(1)).is_ok());
    b.clock.advance(
        GONE.saturating_sub(PING)
            .saturating_sub(Duration::from_secs(1)),
    );
    assert!(channel.ask(&request(2)).is_ok());
    assert_eq!(
        engine.connections(),
        1,
        "inside the bound the line is reused"
    );
    // Every answer renews the silence, so the bound is measured from the last.
    b.clock.advance(GONE.saturating_sub(PING));
    assert!(channel.ask(&request(3)).is_ok());
    assert_eq!(engine.connections(), 2, "at the bound it is not");
}

#[test]
fn pings_are_discarded_wherever_a_frame_is_read_and_keep_the_line_alive() {
    let (listener, at) = listener();
    let b = Bench::new(dead(), Some(presence(at)));
    let script = vec![
        Reply::yes().pinged(2).then(Fate::Pinged(3)),
        Reply::yes().pinged(1),
    ];
    let engine = Engine::listen(b.scratch.path(), listener, script);
    let channel = b.channel();
    assert_eq!(
        channel.ask(&request(1)),
        Ok(vec![Reply::yes().frames[0].clone()])
    );
    // The three pings into the silence are on the socket before the next ask.
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(
        channel.ask(&request(2)),
        Ok(vec![Reply::yes().frames[0].clone()])
    );
    assert_eq!(engine.connections(), 1);
}

#[test]
fn a_line_the_engine_ended_re_enters_the_ladder_however_it_ended() {
    for fate in [Fate::Fin, Fate::Farewell, Fate::Reset, Fate::Garbage] {
        let (listener, at) = listener();
        let b = Bench::new(dead(), Some(presence(at)));
        let script = vec![Reply::yes().then(fate), Reply::yes()];
        let engine = Engine::listen(b.scratch.path(), listener, script);
        let channel = b.channel();
        let first = channel.ask(&request(1));
        assert!(first.is_ok(), "{fate:?}: {first:?}");
        std::thread::sleep(Duration::from_millis(300));
        let again = channel.ask(&request(2));
        assert!(again.is_ok(), "{fate:?}: {again:?}");
        assert_eq!(engine.connections(), 2, "{fate:?}");
    }
}

#[test]
fn a_follower_that_stops_early_drops_its_line() {
    let (listener, at) = listener();
    let b = Bench::new(dead(), Some(presence(at)));
    let many = Reply {
        pings: 0,
        frames: vec![json!({"n": 1}), json!({"n": 2})],
        fate: Fate::Stay,
    };
    let engine = Engine::listen(b.scratch.path(), listener, vec![many, Reply::yes()]);
    let channel = b.channel();
    let mut taken = 0;
    channel
        .follow(&request(1), &mut |_| {
            taken += 1;
            false
        })
        .unwrap();
    assert_eq!(taken, 1);
    assert!(channel.ask(&request(2)).is_ok());
    assert_eq!(
        engine.connections(),
        2,
        "a line with an answer still on it is not reused"
    );
}
