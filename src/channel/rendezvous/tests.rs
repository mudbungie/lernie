//! The two files, their three states, and every derivation held equal to
//! bytes the engine's own code produced.

use super::*;
use crate::test_support::Scratch;

/// The engine's material in the fixture: seed `[1; 32]`, salt `[2; 32]`.
/// Every constant below was printed by yog's `rendezvous::material` and
/// `rendezvous::item` over exactly that pairing.
pub(crate) const ENGINE_PUB: &str =
    "8a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf3748801b40f6f5c";
const PRESENCE_SALT: &str = "832100dd41d9219596850cbb12bb86caabe0d7ebcac788f3dc829b7a2093e55e";
const INBOX_SALT: &str = "5fc17259eeb89d3f93aa22916cecff7a171da3f998f836fb2aaa993b83654b38";
const SEAL_KEY: &str = "c3481ff89e54257cc554538285910e3ab328eccd206cad9a2ad0ed7ddf8b748d";
const INBOX_PUB: &str = "9571b09392c75e738b1829605d29401eaa38adfb63071c7e53259217c7564e58";

pub(crate) fn pairing() -> Pairing {
    Pairing {
        engine: <[u8; 32]>::try_from(unhex(ENGINE_PUB).unwrap()).unwrap(),
        salt: [2u8; 32],
    }
}

/// Lay the fixture pairing down as an entry holds it.
pub(crate) fn provision(dir: &Path) {
    std::fs::write(dir.join(PUBLIC), format!("{ENGINE_PUB}\n")).unwrap();
    std::fs::write(dir.join(SALT), format!("{}\n", hex(&[2u8; 32]))).unwrap();
}

#[test]
fn every_derivation_equals_the_engines() {
    let p = pairing();
    assert_eq!(hex(&p.presence_salt()), PRESENCE_SALT);
    assert_eq!(hex(&p.inbox_salt()), INBOX_SALT);
    assert_eq!(hex(&p.seal_key()), SEAL_KEY);
    assert_eq!(hex(&p.inbox_keypair().unwrap().public()), INBOX_PUB);
}

#[test]
fn nothing_is_none_both_is_a_pairing_and_half_is_a_refusal() {
    let scratch = Scratch::new();
    assert_eq!(read_dir(scratch.path()).unwrap(), None);
    provision(scratch.path());
    assert_eq!(read_dir(scratch.path()).unwrap(), Some(pairing()));
    std::fs::remove_file(scratch.join(SALT)).unwrap();
    let refusal = read_dir(scratch.path()).unwrap_err();
    assert!(
        refusal.contains(PUBLIC) && refusal.contains(SALT) && refusal.contains("ca.pem"),
        "{refusal}"
    );
}

#[test]
fn a_file_that_is_not_32_bytes_of_hex_is_no_material() {
    let scratch = Scratch::new();
    std::fs::write(scratch.join(PUBLIC), "zz\n").unwrap();
    std::fs::write(scratch.join(SALT), "abc\n").unwrap();
    assert_eq!(read_dir(scratch.path()).unwrap(), None);
    std::fs::write(scratch.join(SALT), format!("{}\n", hex(&[7u8; 31]))).unwrap();
    assert_eq!(read_dir(scratch.path()).unwrap(), None);
}

#[test]
fn hex_round_trips_and_refuses_what_is_not_hex() {
    assert_eq!(hex(&[0, 15, 255]), "000fff");
    assert_eq!(unhex("000fff"), Some(vec![0, 15, 255]));
    assert_eq!(unhex("0"), None, "odd length");
    assert_eq!(unhex("0g"), None, "not a digit");
    assert_eq!(unhex("é0"), None, "not even ASCII");
}
