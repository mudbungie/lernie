//! What the call carries, and the port it names: bound once for the run,
//! with the observed address beside the local ones.

use super::*;

#[test]
fn the_punch_port_is_bound_once_for_the_run() {
    let (listener, at) = listener();
    let b = Bench::new(dead(), Some(presence(at)));
    let script = vec![Reply::yes().then(Fate::Fin), Reply::yes()];
    let engine = Engine::listen(b.scratch.path(), listener, script);
    let channel = b.channel();
    assert!(channel.ask(&request(1)).is_ok());
    let (port, asked) = (b.punch_port(), b.node.queries());
    // As if the engine had never been found this run — but the port stands.
    crate::state::worked(&b.scratch.path().display().to_string(), |w| {
        w.endpoints.clear();
    });
    assert!(channel.ask(&request(2)).is_ok());
    assert_eq!(engine.connections(), 2);
    assert!(b.node.queries() > asked, "the commons was asked again");
    assert_eq!(b.punch_port(), port, "the same port, so the peer sees one");
    assert_eq!(
        b.call().map(|c| c.endpoints),
        Some(vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)])
    );
}

#[test]
fn the_call_carries_the_observed_address_at_the_punch_port_once() {
    let seen = |claim: IpAddr| {
        let (listener, at) = listener();
        let b = Bench::seen(
            dead(),
            Some(presence(at)),
            Mood::Claim(SocketAddr::new(claim, 6881)),
        );
        let _engine = Engine::listen(b.scratch.path(), listener, vec![Reply::yes()]);
        assert!(b.channel().ask(&request(1)).is_ok());
        (b.call().map(|c| c.endpoints), b.punch_port())
    };
    let local = |port| SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
    let far: IpAddr = "203.0.113.7".parse().unwrap();
    let (endpoints, port) = seen(far);
    assert_eq!(
        endpoints,
        Some(vec![local(port), SocketAddr::new(far, port)]),
        "the observed address, at the punch port and never the observed one"
    );
    let (endpoints, port) = seen(IpAddr::V4(Ipv4Addr::LOCALHOST));
    assert_eq!(
        endpoints,
        Some(vec![local(port)]),
        "a local address is not said twice"
    );
}
