//! **Register this box** (yog's `docs/REMOTE.md` §15.1 steps 2 to 4; DESIGN
//! §4.50): claim a one-time offer with a key made here, file what comes back
//! as an entry, and show the word the operator approves by.
//!
//! In the order it happens:
//!
//! 1. The URL is read ([`url`]); anything that is not a registration URL
//!    refuses before a socket opens.
//! 2. This box makes a P-256 key pair ([`key`]). The private half is never
//!    sent anywhere.
//! 3. The public half is POSTed to the URL over HTTPS, verified against the
//!    roots the caller passes ([`https`]). Anything but `200` is *this offer
//!    cannot be claimed*: the door answers one bare `404` for a code that is
//!    unknown, expired or already used, and does not say which.
//! 4. The answer and the private key are filed as an entry ([`entry`]),
//!    never over one that exists.
//! 5. The confirmation word is computed from the public key and the CA the
//!    answer carried.
//! 6. The wire is dialled once with the new entry. The server refuses a
//!    certificate nobody has approved — that refusal is expected, and being
//!    heard at all is what marks the offer proved. So an answer of any kind
//!    is *waiting for approval*, and only a dial that reached nothing is said
//!    as a fault, with the entry still filed and the next dial proving it.
//!
//! **This is the one place the seat makes a key, and it never receives one.**
//! The carried envelope it replaces put a private key in a QR symbol; a copy
//! of that symbol was the device.

use std::path::{Path, PathBuf};

use rustls::RootCertStore;
use serde_json::json;

use crate::channel::material::{self, Material};
use crate::cli::Verdict;

/// What the door answered, filed as an entry.
mod entry;
/// The one HTTPS request.
mod https;
/// The key made here, and the confirmation word.
mod key;
/// The registration URL.
mod url;

pub use https::web_roots;

/// The door word, and the word the window's page is opened by.
pub const WORD: &str = "register";

/// What the outcome's line for the confirmation word begins with.
pub const WORD_LINE: &str = "Confirmation word";

/// A box that claimed its offer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registered {
    pub workspace: String,
    /// The name the server knows this box by.
    pub name: String,
    pub grade: String,
    /// The six-character confirmation word.
    pub word: String,
    /// Where the entry was filed.
    pub entry: PathBuf,
    /// What the entry dials, as its `address` file states it.
    pub addresses: Vec<String>,
    /// Whether the server heard this box on the wire: `Err` is why the one
    /// dial reached nothing.
    pub proved: Result<(), String>,
}

impl Registered {
    /// The whole outcome, in the sentences both the command line and the
    /// window say.
    pub fn said(&self) -> String {
        let proved = match &self.proved {
            Ok(()) => "Waiting for approval: the server has heard this box and will refuse it \
                       until the offer is approved."
                .to_owned(),
            Err(why) => format!(
                "The server has not heard this box yet, so the offer cannot be approved: {why}. \
                 The entry is filed; any `lernie` command that reaches that server will do it."
            ),
        };
        format!(
            "Registered as {} ({}) in workspace {}. The entry is filed at {}.\n\
             {WORD_LINE}: {}\n\
             On a seat that is already connected, approve the offer whose row shows that word: \
             `lernie approve {} {}`, or Settings › Servers › offers.\n\
             {proved}",
            self.name,
            self.grade,
            self.workspace,
            self.entry.display(),
            self.word,
            self.workspace,
            self.name
        )
    }
}

/// **Claim the offer at `url`**, verifying its HTTPS against `roots`.
pub fn claim(data_root: &Path, url: &str, roots: RootCertStore) -> Result<Registered, String> {
    let offer = url::parse(url)?;
    let made = key::make()?;
    let body = json!({ "public_key": key::pem("PUBLIC KEY", &made.spki) }).to_string();
    let answer = https::post(&offer, &body, roots)?;
    if answer.status != 200 {
        return Err(format!(
            "this offer cannot be claimed: {} answered {}. The code is unknown, expired or \
             already used — a registration URL works once. Ask for a new offer.",
            offer.authority(),
            answer.status
        ));
    }
    let claimed = entry::read(&answer.body)?;
    let word = key::word(&made.spki, &claimed.ca)?;
    let dir = entry::file(data_root, &claimed, &key::pem("PRIVATE KEY", &made.pkcs8))?;
    Ok(Registered {
        proved: prove(&dir, &claimed.addresses),
        workspace: claimed.workspace,
        name: claimed.name,
        grade: claimed.grade,
        word,
        entry: dir,
        addresses: claimed.addresses,
    })
}

/// **The one dial that proves the offer**: any request at all, down the new
/// entry. Being answered — with the refusal a held certificate earns — is the
/// proof; only reaching nothing is a fault.
fn prove(dir: &Path, addresses: &[String]) -> Result<(), String> {
    let held = Material {
        anchors: dir.join(material::ANCHORS),
        chain: dir.join(material::CHAIN),
        key: dir.join(material::KEY),
        addresses: addresses.to_vec(),
    };
    crate::channel::Channel::open(&held)?
        .ask(&crate::verbs::workspaces())
        .map(|_| ())
        .map_err(|reach| reach.said())
}

/// The command line's `register`: the outcome as a verdict.
pub fn register(data_root: &Path, url: &str, roots: RootCertStore) -> Verdict {
    match claim(data_root, url, roots) {
        Ok(registered) => Verdict::ok(registered.said()),
        Err(refusal) => Verdict::failed(refusal),
    }
}

#[cfg(test)]
mod tests;
