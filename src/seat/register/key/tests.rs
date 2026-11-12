//! The key, its PEM, and the confirmation word against the server's vector.

use super::{base64, make, pem, word};
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{PrivateKeyDer, SubjectPublicKeyInfoDer};

/// yog's own vector (`src/registry/offer/tests.rs`; REMOTE §15.1 step 3): a
/// device public key and an operator CA, both public by construction.
const VECTOR_KEY: &str = "-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAENflj+nLA7lsFuxVfqaryK84n9wP4
3JqG0mf0zqZecVFyPJ6ZX6m9IVK9g0IKk4hdMBbVTWXZvaIRcR4UDpgzCg==
-----END PUBLIC KEY-----
";
pub(crate) const VECTOR_CA: &str = "-----BEGIN CERTIFICATE-----
MIIBdzCCAR2gAwIBAgIUS6jy88xKuMgm0widLw6ejqRjV2EwCgYIKoZIzj0EAwIw
ETEPMA0GA1UEAwwGeW9nLWNhMB4XDTI2MTExMTE5MTgyOVoXDTI5MDIxMzE5MTgy
OVowETEPMA0GA1UEAwwGeW9nLWNhMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE
a2lFtf+Bdi55ENgocMU/VFmRRnrTvKFmGo1Jc/jkpoLCcUiuY+oHUyxbLu9bjGPN
KTzzSgWlmGrNv8lTlWTamKNTMFEwHQYDVR0OBBYEFDuf4kFm3Ug1PoxXdM4G1sUD
nSeYMB8GA1UdIwQYMBaAFDuf4kFm3Ug1PoxXdM4G1sUDnSeYMA8GA1UdEwEB/wQF
MAMBAf8wCgYIKoZIzj0EAwIDSAAwRQIhAIYwai5c3FwmIffCooBMLoSYb8A+40Id
n1/Fr3EXpRUyAiB/98dHbS9USqSz9WaFPMWG6lVZgiWI3ZGtfzdB30JEmA==
-----END CERTIFICATE-----
";

fn vector_spki() -> Vec<u8> {
    SubjectPublicKeyInfoDer::from_pem_slice(VECTOR_KEY.as_bytes())
        .expect("the vector's key")
        .as_ref()
        .to_vec()
}

/// **The word every implementation reproduces**: `2LFTUR`.
#[test]
fn the_confirmation_word_matches_the_servers_vector() {
    assert_eq!(word(&vector_spki(), VECTOR_CA), Ok("2LFTUR".to_owned()));
}

/// Only the FIRST certificate of `ca` is hashed, and a `ca` with none refuses.
#[test]
fn the_word_reads_the_first_certificate_and_refuses_where_there_is_none() {
    let two = format!("{VECTOR_CA}{VECTOR_CA}");
    assert_eq!(word(&vector_spki(), &two), Ok("2LFTUR".to_owned()));
    for empty in [
        "",
        "not pem",
        "-----BEGIN CERTIFICATE-----\n!\n-----END CERTIFICATE-----\n",
    ] {
        let refusal = word(&vector_spki(), empty).expect_err("no certificate");
        assert!(refusal.contains("no CA certificate"), "{refusal}");
    }
}

/// The PEM this box writes is the PEM the vector is: the same 91 bytes come
/// back out as the same text, which pins the base64 and the line length.
#[test]
fn a_public_key_is_written_as_the_pem_the_server_reads() {
    assert_eq!(pem("PUBLIC KEY", &vector_spki()), VECTOR_KEY);
}

/// Every padding case, against RFC 4648's own vectors.
#[test]
fn base64_pads_as_the_standard_does() {
    for (plain, encoded) in [
        ("", ""),
        ("f", "Zg=="),
        ("fo", "Zm8="),
        ("foo", "Zm9v"),
        ("foob", "Zm9vYg=="),
        ("fooba", "Zm9vYmE="),
        ("foobar", "Zm9vYmFy"),
    ] {
        assert_eq!(base64(plain.as_bytes()), encoded);
    }
}

/// **A made key is a P-256 pair in the two encodings the entry and the claim
/// need**: 91 bytes of `SubjectPublicKeyInfo`, and a PKCS#8 private key that
/// rustls reads back out of its PEM. Two makes are two keys.
#[test]
fn a_made_key_is_p256_and_reads_back_as_the_entrys_private_key() {
    let made = make().expect("a key");
    assert_eq!(made.spki.len(), 91);
    assert_eq!(made.spki.get(26), Some(&0x04), "an uncompressed point");
    let text = pem("PRIVATE KEY", &made.pkcs8);
    let read = PrivateKeyDer::from_pem_slice(text.as_bytes()).expect("PKCS#8 PEM");
    assert_eq!(read.secret_der(), made.pkcs8.as_slice());
    let public = pem("PUBLIC KEY", &made.spki);
    assert!(SubjectPublicKeyInfoDer::from_pem_slice(public.as_bytes()).is_ok());
    assert_ne!(make().expect("another").spki, made.spki);
}
