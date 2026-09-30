//! Rung 3, the re-call (yog bl-278f): a dropped line writes one call from
//! the cached presence and punches — no presence read — and a call that
//! expires unanswered clears the cache, so presence is read again. There is
//! no re-punch left to say.

use super::said::{P, refused, written};
use super::*;

/// The entry's cached presence.
fn cached(b: &Bench) -> Vec<SocketAddr> {
    crate::state::worked(&b.scratch.path().display().to_string(), |w| {
        w.endpoints.clone()
    })
}

#[test]
fn a_dropped_line_re_calls_from_cached_presence_and_lands() {
    for (fate, advance, why) in [
        (Fate::Stay, GONE, "past the silence bound"),
        (Fate::Fin, Duration::ZERO, "closed at the far end"),
    ] {
        let (listener, at) = listener();
        let b = Bench::new(dead(), Some(presence(at)));
        let script = vec![Reply::yes().then(fate), Reply::yes()];
        let engine = Engine::listen(b.scratch.path(), listener, script);
        let channel = b.channel();
        assert!(channel.ask(&request(1)).is_ok());
        let (first, seq) = written(&b);
        assert_eq!(cached(&b), [at], "a landed call caches its presence");
        b.said();
        std::thread::sleep(Duration::from_millis(200));
        b.clock.advance(advance);
        assert!(channel.ask(&request(2)).is_ok());
        assert_eq!(engine.connections(), 2, "the dropped line was not reused");
        let (nonce, again) = written(&b);
        assert_ne!(nonce, first, "a fresh call");
        assert!(again > seq, "a rising sequence");
        assert_eq!(
            b.said(),
            [
                format!("{P} held line dropped — {why}"),
                refused(),
                format!(
                    "{P} re-call from cached presence — nonce {nonce}, seq {again}, 1 endpoint(s) (1 v4), 1 ack(s)"
                ),
                format!("{P} punch for nonce {nonce} started — 1 endpoint(s) (1 v4), window 3s"),
                format!("{P} punch for nonce {nonce} landed (1 v4)"),
                format!("{P} punched line held between asks"),
            ],
            "no presence read, and no re-punch"
        );
    }
}

#[test]
fn an_expired_re_call_clears_the_cache_and_presence_is_read_again() {
    let (listener, at) = listener();
    let b = Bench::new(dead(), Some(presence(at)));
    let script = vec![Reply::yes().then(Fate::Fin), Reply::yes()];
    let _engine = Engine::listen(b.scratch.path(), listener, script);
    let channel = b.channel().tuned(
        Roving {
            window: Duration::from_millis(500),
            ..b.roving()
        },
        b.clock.arc(),
    );
    assert!(channel.ask(&request(1)).is_ok());
    // As if the engine had moved since: the cache names where it no longer is.
    crate::state::worked(&b.scratch.path().display().to_string(), |w| {
        w.endpoints = vec![dead()];
    });
    b.said();
    std::thread::sleep(Duration::from_millis(200));
    assert!(channel.ask(&request(2)).is_ok());
    let said = b.said();
    let recall = said.get(2).unwrap();
    assert!(
        recall.starts_with(&format!("{P} re-call from cached presence — nonce ")),
        "{said:?}"
    );
    let expired = said.get(4).unwrap();
    assert!(
        expired.ends_with(" expired after 500ms with no stream"),
        "{said:?}"
    );
    assert_eq!(
        said.get(5).unwrap(),
        &format!("{P} presence read — seq 1, 1 endpoint(s) (1 v4)"),
        "the cleared cache sends the climb to presence"
    );
    assert_eq!(cached(&b), [at], "and the call that landed caches it anew");
}

#[test]
fn a_call_that_expires_leaves_no_cache_so_the_next_climb_reads_presence() {
    let b = Bench::new(dead(), Some(presence(dead())));
    let channel = b.channel().tuned(
        Roving {
            window: Duration::from_millis(300),
            ..b.roving()
        },
        b.clock.arc(),
    );
    assert!(channel.ask(&request(1)).is_err());
    assert_eq!(cached(&b), Vec::<SocketAddr>::new());
    b.said();
    assert!(channel.ask(&request(2)).is_err());
    let said = b.said();
    assert!(
        !said.iter().any(|line| line.contains("re-call")),
        "{said:?}"
    );
    assert_eq!(
        said.get(1).unwrap(),
        &format!("{P} presence read — seq 1, 1 endpoint(s) (1 v4)")
    );
}

#[test]
fn a_re_call_carries_where_the_last_walk_saw_this_end() {
    let (listener, at) = listener();
    let far: IpAddr = "203.0.113.7".parse().unwrap();
    let b = Bench::seen(
        dead(),
        Some(presence(at)),
        Mood::Claim(SocketAddr::new(far, 6881)),
    );
    let script = vec![Reply::yes().then(Fate::Fin), Reply::yes()];
    let _engine = Engine::listen(b.scratch.path(), listener, script);
    let channel = b.channel();
    assert!(channel.ask(&request(1)).is_ok());
    std::thread::sleep(Duration::from_millis(200));
    assert!(channel.ask(&request(2)).is_ok());
    let port = b.punch_port();
    assert_eq!(
        b.call().map(|c| c.endpoints),
        Some(vec![
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port),
            SocketAddr::new(far, port),
        ]),
        "the re-call walked nowhere before writing, yet names the observed address"
    );
}
