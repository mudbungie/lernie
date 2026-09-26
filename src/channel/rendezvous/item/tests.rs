//! Both items open from bytes the ENGINE sealed — cross-implementation
//! compatibility asserted, not assumed — round-trip through this end's own
//! seal, and everything that is not one of them opens to nothing.

use super::super::tests::pairing;
use super::super::unhex;
use super::*;
use crate::dht::{Mutable, target_of};

/// Sealed by yog's `Presence::seal` under the fixture pairing's seal key,
/// listing [`endpoints`].
const PRESENCE_SEALED: &str = "67d248c53ff4a2cfc38eccfc6908a59cd9f9c40d8c99848ef13f20f8dc7312a093fd7e7b9688c0f7bbd5b5b6ac692037db7b4cbe08ece3";
/// Signed by yog's `Keypair::sign` under the fixture seed, presence salt,
/// `seq` 5, over [`PRESENCE_SEALED`].
const PRESENCE_SIG: &str = "ac2e3a817dbca5ba020cee7d2d5983e03130ab3b5c4c93f0c8ee30ffdf1809a00e8e086a18ca5eadf8b3079657ac22fee01dbeae720e3f24bc91c80456496b02";
const PRESENCE_TARGET: &str = "76d0bf72a8c6a1910af4b0e3efe2cee463891b9d";
/// Sealed by yog's `Call::seal`: nonce `0x0102030405060708`, [`endpoints`].
const CALL_SEALED: &str = "ab8c97a9391ca2edea1d992e1e431840ae0df9db333cb19a7419a0c21e3a64bc63843cf27713ac36fa716909641f8890bdc46d715c417931efc93afe596368";
/// Signed under the DERIVED inbox keypair, inbox salt, `seq` 7.
const CALL_SIG: &str = "3e329f43c64614f5ba6947954050514c46bb90bebedbc229af9707c5f36b7dfb4c08d3b142f500c73f73a6e41ba699213bab17dec7aed2e105149f5e4baecd0f";
const CALL_TARGET: &str = "eda6251eef41246292c5d03ba6c51defd0c07ec4";

const KEY: [u8; 32] = [9u8; 32];

fn endpoints() -> Vec<SocketAddr> {
    vec![
        "127.0.0.1:7737".parse().unwrap(),
        "[::1]:7738".parse().unwrap(),
    ]
}

fn sig(hex: &str) -> [u8; 64] {
    <[u8; 64]>::try_from(unhex(hex).unwrap()).unwrap()
}

#[test]
fn presence_the_engine_sealed_and_signed_opens_and_verifies_here() {
    let p = pairing();
    let sealed = unhex(PRESENCE_SEALED).unwrap();
    assert_eq!(
        Presence::open(&p.seal_key(), &sealed),
        Some(Presence {
            endpoints: endpoints()
        })
    );
    let item = Mutable {
        key: p.engine,
        salt: p.presence_salt(),
        seq: 5,
        value: sealed,
        sig: sig(PRESENCE_SIG),
    };
    assert!(item.verify(), "the engine's signature holds under its key");
    assert_eq!(item.target().to_string(), PRESENCE_TARGET);
    assert_eq!(
        target_of(&p.engine, &p.presence_salt()).to_string(),
        PRESENCE_TARGET
    );
}

#[test]
fn a_call_the_engine_sealed_and_signed_is_what_this_end_would_write() {
    let p = pairing();
    let sealed = unhex(CALL_SEALED).unwrap();
    assert_eq!(
        Call::open(&p.seal_key(), &sealed),
        Some(Call {
            nonce: 0x0102_0304_0506_0708,
            endpoints: endpoints()
        })
    );
    let item = Mutable {
        key: p.inbox_keypair().unwrap().public(),
        salt: p.inbox_salt(),
        seq: 7,
        value: sealed.clone(),
        sig: sig(CALL_SIG),
    };
    assert!(item.verify(), "the derived inbox keypair is the engine's");
    assert_eq!(item.target().to_string(), CALL_TARGET);
    // And this end signs the same bytes to the same signature: ed25519 is
    // deterministic, so the derived keypair is proved equal by its output.
    let ours = p
        .inbox_keypair()
        .unwrap()
        .sign(p.inbox_salt(), 7, sealed)
        .unwrap();
    assert_eq!(ours.sig, sig(CALL_SIG));
}

#[test]
fn a_call_this_end_seals_opens_under_the_engines_half() {
    let call = Call {
        nonce: 0x0102_0304_0506_0708,
        endpoints: endpoints(),
    };
    let sealed = call.seal(&KEY).unwrap();
    assert_eq!(sealed.len(), 12 + 8 + 1 + 7 + 19 + 16);
    assert_eq!(Call::open(&KEY, &sealed), Some(call));
    assert!(
        !sealed.windows(4).any(|w| w == [127, 0, 0, 1]),
        "the address is not readable on the commons"
    );
    assert_eq!(
        Call::open(&[8u8; 32], &sealed),
        None,
        "another key opens nothing"
    );
}

#[test]
fn presence_round_trips_through_the_stand_in_seal() {
    for endpoints in [endpoints(), Vec::new()] {
        let presence = Presence { endpoints };
        let sealed = presence.seal(&KEY).unwrap();
        assert_eq!(Presence::open(&KEY, &sealed), Some(presence));
    }
}

#[test]
fn a_sealed_item_of_one_kind_is_not_the_other() {
    let presence = Presence {
        endpoints: endpoints(),
    };
    let sealed = presence.seal(&KEY).unwrap();
    assert_eq!(Call::open(&KEY, &sealed), None, "no nonce to read");
    let call = Call {
        nonce: 1,
        endpoints: endpoints(),
    };
    let sealed = call.seal(&KEY).unwrap();
    assert_eq!(
        Presence::open(&KEY, &sealed),
        None,
        "a count byte of 0 then bytes left over"
    );
}

#[test]
fn what_will_not_decode_is_nothing() {
    assert_eq!(Presence::open(&KEY, b"short"), None, "no nonce");
    assert_eq!(Presence::open(&KEY, &[0u8; 40]), None, "no tag");
    for plain in [
        vec![],
        vec![1],
        vec![1, 5, 0, 0],
        vec![1, 4, 1, 2, 3],
        vec![1, 4, 1, 2, 3, 4, 0],
        vec![1, 6, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0],
    ] {
        let sealed = seal(&KEY, &plain).unwrap();
        assert_eq!(Presence::open(&KEY, &sealed), None, "{plain:?}");
    }
    let sealed = seal(&KEY, &[0, 0, 0, 0, 0, 0, 0, 1, 0]).unwrap();
    assert_eq!(
        Call::open(&KEY, &sealed),
        Some(Call {
            nonce: 1,
            endpoints: Vec::new()
        })
    );
}

#[test]
fn the_list_is_bounded_at_a_byte() {
    let many: Vec<SocketAddr> = (0..300u16)
        .map(|port| SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port))
        .collect();
    let (decoded, rest) = decode(&encode(&many)).unwrap();
    assert_eq!(decoded.len(), 255);
    assert!(rest.is_empty());
}
