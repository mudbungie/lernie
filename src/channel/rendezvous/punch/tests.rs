//! The punch on loopback: a SYN that lands on the peer's listener, the
//! peer's SYN landing here, and nobody there.

use super::*;
use std::io::{Read, Write};

fn loopback(port: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
}

#[test]
fn a_punch_lands_on_the_peers_listener_and_carries_bytes() {
    let engine = TcpListener::bind("127.0.0.1:0").unwrap();
    let target = engine.local_addr().unwrap();
    let served = std::thread::spawn(move || {
        let (mut stream, _) = engine.accept().unwrap();
        let mut byte = [0u8; 1];
        stream.read_exact(&mut byte).unwrap();
        byte[0]
    });
    let client = Punch::bind(0).unwrap();
    let mut stream = client.punch(vec![target], Duration::from_secs(5)).unwrap();
    assert_eq!(
        stream.local_addr().unwrap().port(),
        client.port(),
        "the SYN left from the fixed port"
    );
    stream.write_all(b"x").unwrap();
    assert_eq!(served.join().unwrap(), b'x');
}

#[test]
fn the_peers_syn_lands_on_this_listener_while_ours_goes_nowhere() {
    let gone = TcpListener::bind("127.0.0.1:0").unwrap();
    let nowhere = gone.local_addr().unwrap();
    drop(gone);
    let client = Punch::bind(0).unwrap();
    let here = loopback(client.port());
    let peer = std::thread::spawn(move || {
        let mut stream = TcpStream::connect(here).unwrap();
        stream.write_all(b"y").unwrap();
    });
    let mut stream = client.punch(vec![nowhere], Duration::from_secs(5)).unwrap();
    let mut byte = [0u8; 1];
    stream.read_exact(&mut byte).unwrap();
    assert_eq!(byte[0], b'y');
    peer.join().unwrap();
}

#[test]
fn nobody_there_is_no_stream_at_the_end_of_the_window() {
    let gone = TcpListener::bind("127.0.0.1:0").unwrap();
    let target = gone.local_addr().unwrap();
    drop(gone);
    let punch = Punch::bind(0).unwrap();
    let started = Instant::now();
    assert!(
        punch
            .punch(vec![target], Duration::from_millis(400))
            .is_none()
    );
    assert!(
        started.elapsed() >= Duration::from_millis(400),
        "the window was waited out"
    );
}

#[test]
fn a_port_nothing_can_bind_refuses() {
    let refusal = Punch::bind(1).err().unwrap();
    assert!(refusal.contains("punch"), "{refusal}");
}

#[test]
fn v6_is_punched_first() {
    let v4: SocketAddr = "192.0.2.4:1".parse().unwrap();
    let v6: SocketAddr = "[2001:db8::1]:1".parse().unwrap();
    assert_eq!(ordered(vec![v4, v6, v4]), vec![v6, v4, v4]);
}

#[test]
fn a_v6_loopback_punch_lands_where_the_box_has_v6() {
    let engine = Punch::bind(0).unwrap();
    let target: SocketAddr = format!("[::1]:{}", engine.port()).parse().unwrap();
    let has_v6 = engine.listeners.len() == 2;
    let client = Punch::bind(0).unwrap();
    let served = std::thread::spawn(move || engine.punch(vec![], Duration::from_millis(600)));
    let landed = client.punch(vec![target], Duration::from_millis(500));
    assert_eq!(
        landed.is_some(),
        has_v6,
        "a v6 SYN lands exactly where a v6 listener is"
    );
    assert_eq!(served.join().unwrap().is_some(), has_v6);
}

#[test]
fn the_box_advertises_no_loopback() {
    let ips = local_ips();
    assert!(ips.len() <= 2);
    assert!(ips.iter().all(|ip| !ip.is_loopback()));
}
