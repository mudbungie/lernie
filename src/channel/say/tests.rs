//! The lines themselves: each says counts, seqs, nonces and families, and no
//! line built from addresses ever carries one.

use super::*;
use std::net::{Ipv4Addr, Ipv6Addr};

fn endpoints() -> Vec<SocketAddr> {
    vec![
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 7)), 7737),
        SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 7738),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 7739),
    ]
}

fn ips(of: &[&str]) -> Vec<IpAddr> {
    of.iter().map(|ip| ip.parse().unwrap()).collect()
}

/// A v4 address built from its octets — the shared space is not a
/// documentation range, so it is never written dotted.
fn v4(a: u8, b: u8) -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(a, b, 0, 1))
}

#[test]
fn every_line_is_the_house_shape_and_names_no_address() {
    let w = Duration::from_secs(40);
    let lines = [
        taken(),
        dropped("past the silence bound"),
        direct(Ok(()), w),
        direct(Err(ErrorKind::TimedOut), Duration::from_secs(5)),
        direct(Err(ErrorKind::ConnectionRefused), w),
        no_bootstrap(),
        withheld("presence not read"),
        no_presence(),
        unopened(4),
        presence(9, &endpoints()),
        overlay(&[v4(100, 64), ips(&["fd7a:115c:a1e0::1"])[0]]).unwrap(),
        call(7, 9, &endpoints(), 2),
        started("punch for nonce 7", &endpoints(), w),
        landed("punch for nonce 7", Some(IpAddr::V6(Ipv6Addr::LOCALHOST))),
        expired("re-punch", w),
        kept(),
        pinged(),
    ];
    for line in &lines {
        assert!(line.starts_with("lernie: rendezvous: "), "{line}");
        for addr in ["203.0.113", "127.0.0.1", "::1", "100.64", "fd7a"] {
            assert!(!line.contains(addr), "{line} names {addr}");
        }
    }
    assert_eq!(
        call(7, 9, &endpoints(), 2),
        "lernie: rendezvous: call nonce 7 written — seq 9, 3 endpoint(s) (1 v6, 2 v4), 2 ack(s)"
    );
    assert_eq!(
        direct(Err(ErrorKind::TimedOut), Duration::from_secs(5)),
        "lernie: rendezvous: direct rung timed out after 5s"
    );
    assert_eq!(
        direct(Err(ErrorKind::ConnectionRefused), w),
        "lernie: rendezvous: direct rung refused — connection refused"
    );
    assert_eq!(
        landed("re-punch", None),
        "lernie: rendezvous: re-punch landed (none)"
    );
}

#[test]
fn families_count_v6_first_and_say_none_for_nothing() {
    assert_eq!(families(&[]), "none");
    assert_eq!(families(&ips(&["::1", "127.0.0.1", "::2"])), "2 v6, 1 v4");
    assert_eq!(families(&ips(&["127.0.0.1"])), "1 v4");
}

#[test]
fn the_overlay_line_is_said_only_when_every_address_is_one() {
    let ula = ips(&["fd00::1"])[0];
    for all in [
        vec![v4(100, 64)],
        vec![v4(100, 127), ula],
        ips(&["fc00::1"]),
    ] {
        assert!(overlay(&all).is_some(), "{all:?}");
    }
    let doc = ips(&["203.0.113.7"])[0];
    for not in [
        vec![],
        vec![v4(100, 64), doc],
        vec![v4(100, 128)],
        vec![v4(100, 63)],
        vec![v4(99, 64)],
        ips(&["fe80::1"]),
        ips(&["2001:db8::1"]),
    ] {
        assert_eq!(overlay(&not), None, "{not:?}");
    }
}

#[test]
fn the_default_sink_writes_a_line_and_debugs_as_its_name() {
    let say = Say::default();
    say.line("lernie: rendezvous: a line the suite says on stderr");
    assert_eq!(format!("{say:?}"), "Say");
}
