//! **One HTTPS POST, and nothing else HTTP** (yog's `docs/REMOTE.md` §15.1
//! step 2, §15.2; DESIGN §4.50).
//!
//! The registration door speaks exactly one request, so this is exactly one
//! request: HTTP/1.1 over `rustls`, a JSON body with a `Content-Length`,
//! `Connection: close`, and a bounded read of what comes back. No HTTP
//! library is linked for it and none may be — a client that can follow a
//! redirect, keep a connection or negotiate an encoding is a capability a
//! seat has no other use for.
//!
//! **The trust roots are a parameter.** The door is `https` and is verified
//! against the PUBLIC web's roots, never the operator CA: the box holds no
//! operator CA yet, which is what it is registering to get. Production
//! passes [`web_roots`]; the suite passes the root of its own local server.
//! This configuration is built for the one request and dropped — the wire's
//! mTLS configuration ([`crate::channel::tls`]) is another object and is
//! untouched.
//!
//! **Bounded three ways**: the connect ([`crate::channel::connect`]'s five
//! seconds per address), each read ([`QUIET`]), and the answer's size
//! ([`MOST`]). A reply past the size is refused rather than truncated,
//! because half a certificate is not a smaller certificate.

use std::io::{Read, Write};
use std::sync::Arc;
use std::time::Duration;

use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};

use super::url::Offer;

/// How long one read may find nothing.
const QUIET: Duration = Duration::from_secs(15);
/// The most an answer may be, head and body together. The real one is a CA
/// and a leaf — about two kilobytes.
const MOST: usize = 64 * 1024;

/// What the door answered: the status, and the body where there was one.
#[derive(Debug, PartialEq, Eq)]
pub struct Answer {
    pub status: u16,
    pub body: Vec<u8>,
}

/// **The public web's trust roots** — Mozilla's set, as `webpki-roots`
/// carries it. The one dependency approved for this act (`Cargo.toml`).
pub fn web_roots() -> RootCertStore {
    RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    }
}

/// POST `body` as JSON to the offer's URL, verifying the server against
/// `roots`, and answer what it said.
pub fn post(offer: &Offer, body: &str, roots: RootCertStore) -> Result<Answer, String> {
    let at = offer.authority();
    let config =
        ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .map_err(|e| format!("tls versions: {e}"))?
            .with_root_certificates(roots)
            .with_no_client_auth();
    let name = ServerName::try_from(offer.host.clone())
        .map_err(|e| format!("{at}: not a server name: {e}"))?;
    let tcp = crate::channel::connect(&format!("{}:{}", offer.host(), offer.port))
        .and_then(|tcp| tcp.set_read_timeout(Some(QUIET)).map(|()| tcp))
        .map_err(|e| format!("connect {at}: {e}"))?;
    let conn =
        ClientConnection::new(Arc::new(config), name).map_err(|e| format!("tls {at}: {e}"))?;
    let mut tls = StreamOwned::new(conn, tcp);
    let request = format!(
        "POST {} HTTP/1.1\r\nHost: {at}\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        offer.path(),
        body.len()
    );
    tls.write_all(request.as_bytes())
        .map_err(|e| format!("{at} could not be spoken to over HTTPS: {e}"))?;
    let mut said = Vec::new();
    let mut chunk = [0u8; 4096];
    // The read ends at the answer's stated length, at the peer's close, or at
    // the first error — a peer that hangs up without TLS's own goodbye has
    // still said whatever it said, and `read` judges that.
    while read(&said).is_none() {
        match tls.read(&mut chunk) {
            Ok(n) if n > 0 => said.extend_from_slice(chunk.get(..n).unwrap_or_default()),
            _ => break,
        }
        if said.len() > MOST {
            return Err(format!(
                "{at} answered more than {MOST} bytes; refused unread"
            ));
        }
    }
    read(&said)
        .or_else(|| ended(&said))
        .ok_or_else(|| format!("{at} did not answer as HTTP"))
}

/// The answer, once the whole of it is in: a head, and as many body bytes as
/// its `Content-Length` states. `None` while more is owed, and for an answer
/// that states no length.
fn read(said: &[u8]) -> Option<Answer> {
    let (status, head, body) = split(said)?;
    let length: usize = head
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, value)| value.trim().parse().ok())?;
    Some(Answer {
        status,
        body: body.get(..length)?.to_vec(),
    })
}

/// The answer as the peer's close left it: whatever followed the head.
fn ended(said: &[u8]) -> Option<Answer> {
    let (status, _, body) = split(said)?;
    Some(Answer {
        status,
        body: body.to_vec(),
    })
}

/// The status, the head's text and the bytes after the blank line.
fn split(said: &[u8]) -> Option<(u16, &str, &[u8])> {
    let end = said.windows(4).position(|four| four == b"\r\n\r\n")?;
    let head = std::str::from_utf8(said.get(..end)?).ok()?;
    let status = head
        .strip_prefix("HTTP/1.")?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()?;
    Some((status, head, said.get(end + 4..)?))
}

#[cfg(test)]
mod tests;
