//! **A registration door** (yog's `docs/REMOTE.md` §15.2): a TLS listener
//! that asks for no client certificate, reads one HTTP request and answers
//! whatever the test scripted. The far end of `crate::seat::register`.
//!
//! It presents the mint's server leaf, so the root a test injects is the
//! mint's CA — which is what stands in for the public web's roots.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc;

use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::{RootCertStore, ServerConfig, ServerConnection, StreamOwned};

use crate::channel::{material, tls};

/// A door, and every request it was handed.
pub(crate) struct Door {
    /// `https://127.0.0.1:<port>`, the head of every URL at this door.
    origin: String,
    requests: mpsc::Receiver<String>,
}

impl Door {
    /// Stand one up over the material in `dir`, answering each connection
    /// with what `answer` makes of the request's text.
    pub(crate) fn start(dir: &Path, answer: impl Fn(&str) -> Vec<u8> + Send + 'static) -> Door {
        let read = |leaf: &str| dir.join(format!("{}.{leaf}", super::mint::SERVER));
        let chain: Vec<CertificateDer<'static>> = CertificateDer::pem_file_iter(read("pem"))
            .expect("the server chain")
            .collect::<Result<_, _>>()
            .expect("the server chain");
        let key = PrivateKeyDer::from_pem_file(read("key")).expect("the server key");
        let config = Arc::new(
            ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
                .with_safe_default_protocol_versions()
                .expect("tls versions")
                .with_no_client_auth()
                .with_single_cert(chain, key)
                .expect("the server identity"),
        );
        let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback port");
        let origin = format!("https://{}", listener.local_addr().expect("bound"));
        let (heard, requests) = mpsc::channel();
        std::thread::spawn(move || {
            for tcp in listener.incoming().flatten() {
                let conn = ServerConnection::new(Arc::clone(&config)).expect("the door's config");
                let mut tls = StreamOwned::new(conn, tcp);
                let request = request(&mut tls);
                let _ = tls.write_all(&answer(&request));
                tls.conn.send_close_notify();
                let _ = tls.flush();
                // A test that has stopped listening is not this thread's fault.
                let _ = heard.send(request);
            }
        });
        Door { origin, requests }
    }

    /// The registration URL for `code` at this door.
    pub(crate) fn url(&self, code: &str) -> String {
        format!("{}/yog/register/{code}", self.origin)
    }

    /// The next request it was handed, whole, as text.
    pub(crate) fn heard(&self) -> String {
        self.requests
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("a request reached the door")
    }

    /// The root a claim of this door verifies against.
    pub(crate) fn roots(dir: &Path) -> RootCertStore {
        tls::anchors(&dir.join(material::ANCHORS)).expect("the mint's CA")
    }
}

/// One request: the head, then as many body bytes as it states. A peer that
/// never completes a handshake leaves this with nothing.
fn request(tls: &mut impl Read) -> String {
    let mut read = Vec::new();
    let mut chunk = [0u8; 1024];
    loop {
        let text = String::from_utf8_lossy(&read).into_owned();
        if let Some((head, body)) = text.split_once("\r\n\r\n") {
            let owed = head
                .lines()
                .find_map(|line| line.strip_prefix("Content-Length: "))
                .and_then(|n| n.parse().ok())
                .unwrap_or(0);
            if body.len() >= owed {
                return text;
            }
        }
        match tls.read(&mut chunk) {
            Ok(n) if n > 0 => read.extend_from_slice(&chunk[..n]),
            _ => return text,
        }
    }
}

/// A `200` carrying `body` as JSON, the way the server's door says it.
pub(crate) fn ok(body: &serde_json::Value) -> Vec<u8> {
    let body = body.to_string();
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\
         Connection: close\r\n\r\n{body}",
        body.len()
    )
    .into_bytes()
}

/// The door's one refusal.
pub(crate) const NOT_FOUND: &[u8] =
    b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
