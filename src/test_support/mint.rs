//! **The operator's act, performed by the suite** (yog's `docs/REMOTE.md`
//! §1.4).
//!
//! The seat mints nothing. A certificate and its key are issued by the
//! operator's own CA on the box that holds it and carried to the seat by hand —
//! so there is no production caller here, and there must never be one: a seat
//! that could provision itself over the wire would be a seat any wire could
//! provision. What the suite needs is a channel to test, and the honest way to
//! get one is to do out of band exactly what the operator does out of band.
//!
//! It shells to `openssl`, which is the tool an operator uses and the tool yog's
//! own mint recipe uses. **This file is the crate's one spawn site**
//! (`rules/no-bare-command.yml`, `rules/no-bare-fork.yml`): the seat forks
//! nothing in production, so the confinement rules name the one place that
//! does. Nothing is committed — a fixture key in a tree is a private key in a
//! repository.
//!
//! **What each leaf says**, and every fact here is REMOTE's rather than this
//! file's:
//!
//! - the **engine** leaf carries `serverAuth` and a SAN naming loopback, which
//!   is what a client verifies against what it dialled (REMOTE §8: the server
//!   leaf always carries `IP:127.0.0.1`, because a window is a client of
//!   loopback unconditionally);
//! - the **seat** leaf carries `clientAuth` and no organizational unit, which
//!   is operator grade — REMOTE §4.2's default-operator rule, where a subject
//!   that says nothing about a grade is the whole boundary.

use std::path::Path;
use std::process::Command;

/// The CA's own key. It never leaves the scratch directory, and no production
/// path knows the name.
const CA_KEY: &str = "ca.key";
/// The curve every key is drawn on.
const CURVE: &str = "ec_paramgen_curve:P-256";
/// How long a minted certificate is good for. A day, because the longest thing
/// the suite does with one is finish.
const DAYS: &str = "1";
/// The common name the suite's seat presents.
pub(crate) const SEAT_NAME: &str = "lernie-test-seat";
/// The engine leaf's basename and common name.
pub(crate) const ENGINE: &str = "engine";

/// Mint a CA and both leaves into `dir`, in the layout
/// [`material`](crate::channel::material) reads: the seat's pair is written as
/// `client.pem`/`client.key`, which is the spelling REMOTE §8.2 fixes for a
/// client-side directory.
pub(crate) fn material(dir: &Path) {
    std::fs::create_dir_all(dir).expect("the scratch directory");
    ca(dir);
    leaf(
        dir,
        ENGINE,
        &format!("/CN={ENGINE}"),
        "IP:127.0.0.1",
        "serverAuth",
    );
    leaf(
        dir,
        "client",
        &format!("/CN={SEAT_NAME}"),
        &format!("DNS:{SEAT_NAME}"),
        "clientAuth",
    );
}

/// [`material`], plus the address file — the whole of what one provisioned
/// channel holds, answered as the material a channel opens from. The tests that
/// stand an [`Engine`](super::engine::Engine) up do not use it: an engine binds
/// a kernel-chosen port and writes the address itself, because only the
/// listener knows what `:0` became.
pub(crate) fn provisioned(dir: &Path, address: &str) -> crate::channel::material::Material {
    material(dir);
    std::fs::write(dir.join(crate::channel::material::ADDRESS), address).expect("the address");
    crate::channel::material::read_dir(dir)
        .expect("readable")
        .expect("provisioned")
}

/// The self-signed operator CA both ends verify against.
fn ca(dir: &Path) {
    tool(&[
        "req",
        "-x509",
        "-newkey",
        "ec",
        "-pkeyopt",
        CURVE,
        "-nodes",
        "-sha256",
        "-days",
        DAYS,
        "-subj",
        "/CN=lernie-test-ca",
        "-keyout",
        &path(dir, CA_KEY),
        "-out",
        &path(dir, crate::channel::material::ANCHORS),
    ]);
}

/// One leaf: a key and a bare request, then the signature that carries the two
/// facts the issuer decides — the subject alternative name and the extended key
/// usage.
///
/// The extensions travel in a file rather than through `req -addext` with
/// `x509 -copy_extensions`, which is OpenSSL-only: LibreSSL ships as `openssl`
/// on macOS and refuses that flag outright. `-extfile`/`-extensions` is the
/// spelling both toolsets have, and it is the more honest model besides — what
/// a certificate asserts is decided by whoever signs it, not by whoever asked.
fn leaf(dir: &Path, name: &str, subject: &str, san: &str, eku: &str) {
    let ext = dir.join(format!("{name}.ext"));
    std::fs::write(
        &ext,
        format!("[leaf]\nsubjectAltName={san}\nextendedKeyUsage={eku}\n"),
    )
    .expect("the extension file");
    tool(&[
        "req",
        "-new",
        "-newkey",
        "ec",
        "-pkeyopt",
        CURVE,
        "-nodes",
        "-sha256",
        "-subj",
        subject,
        "-keyout",
        &path(dir, &format!("{name}.key")),
        "-out",
        &path(dir, &format!("{name}.csr")),
    ]);
    tool(&[
        "x509",
        "-req",
        "-sha256",
        "-days",
        DAYS,
        "-extfile",
        &ext.to_string_lossy(),
        "-extensions",
        "leaf",
        "-in",
        &path(dir, &format!("{name}.csr")),
        "-CA",
        &path(dir, crate::channel::material::ANCHORS),
        "-CAkey",
        &path(dir, CA_KEY),
        // LibreSSL refuses to sign with no serial file unless told it may make
        // one; OpenSSL 3 accepts the flag and does the same thing.
        "-CAcreateserial",
        "-out",
        &path(dir, &format!("{name}.pem")),
    ]);
}

/// **Re-mint this box's own pair as FOOT grade**, over the operator-grade one
/// [`material`] laid down — REMOTE §4.2's misconfiguration, performed the way
/// an operator performs it by accident: one `-subj`, one attribute longer. The
/// word is the CA's here and the reader's in [`leaf`](crate::channel::leaf);
/// they are two parties, and the suite is what proves they agree.
///
/// `named` is whether the subject states a common name at all. A leaf that says
/// foot and names nobody is exactly as wrong, and it is the only way to reach
/// the half of the sentence that has no client to quote.
pub(crate) fn foot(dir: &Path, named: bool) {
    let subject = if named {
        format!("/CN={SEAT_NAME}/OU=foot")
    } else {
        "/OU=foot".to_owned()
    };
    leaf(
        dir,
        "client",
        &subject,
        &format!("DNS:{SEAT_NAME}"),
        "clientAuth",
    );
}

/// One path, as the string `openssl` takes.
fn path(dir: &Path, leaf: &str) -> String {
    dir.join(leaf).to_string_lossy().into_owned()
}

/// One `openssl` run. A failure is the scaffolding's own, so it dies here with
/// the tool's sentence rather than becoming a confusing refusal three layers up.
fn tool(args: &[&str]) {
    let said = Command::new("openssl")
        .args(args)
        .output()
        .expect("openssl is installed");
    assert!(
        said.status.success(),
        "openssl {args:?}: {}",
        String::from_utf8_lossy(&said.stderr)
    );
}
