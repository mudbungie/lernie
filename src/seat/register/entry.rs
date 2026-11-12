//! **What the door answered, filed as an entry** (yog's `docs/REMOTE.md`
//! §15.1 step 2; DESIGN §4.50).
//!
//! The answer is one JSON object — `workspace`, `name`, `grade`, `address`
//! (the list a client dials, in order), `ca` and `cert` — and with the key
//! this box made it is a whole entry: `ca.pem`, `client.pem`, `client.key`
//! and `address`, under `<data root>/wire/workspaces/<workspace>/`, in
//! [`crate::channel::material`]'s own names.
//!
//! **An entry that already exists is never written over.** The directory is
//! made with `create_dir`, which the filesystem performs for exactly one
//! caller, so there is no check to race and no file of an existing entry is
//! opened. A second certificate under one directory would leave the key of
//! one beside the leaf of the other.
//!
//! **The workspace name comes off the network and becomes a path**, so it is
//! read as exactly one ordinary path component or refused: a name with a
//! separator in it, or `..`, files nothing.

use std::path::{Component, Path, PathBuf};

use serde_json::Value;

use crate::channel::entries;
use crate::channel::material::{ADDRESS, ANCHORS, CHAIN, KEY};

/// What the server issued this box.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claimed {
    pub workspace: String,
    pub name: String,
    pub grade: String,
    /// The endpoints to dial, in order. Never empty.
    pub addresses: Vec<String>,
    /// The operator CA, PEM.
    pub ca: String,
    /// The leaf signed for this box's key, PEM.
    pub cert: String,
}

/// Read the door's answer, or name the first field it does not carry.
pub fn read(body: &[u8]) -> Result<Claimed, String> {
    let answer: Value = serde_json::from_slice(body).unwrap_or(Value::Null);
    let text = |key: &str| {
        answer
            .get(key)
            .and_then(Value::as_str)
            .filter(|said| !said.is_empty())
            .map(str::to_owned)
            .ok_or_else(|| missing(key))
    };
    let addresses: Vec<String> = answer
        .get(ADDRESS)
        .and_then(Value::as_array)
        .map(|listed| listed.iter().filter_map(Value::as_str).collect::<Vec<_>>())
        .map(|listed| crate::channel::material::list(&listed.join("\n")))
        .filter(|listed| !listed.is_empty())
        .ok_or_else(|| missing(ADDRESS))?;
    let workspace = text("workspace")?;
    let mut parts = Path::new(&workspace).components();
    if !matches!((parts.next(), parts.next()), (Some(Component::Normal(leaf)), None) if leaf == workspace.as_str())
    {
        return Err(format!(
            "the server named the workspace {workspace:?}, which cannot be a directory name here; \
             nothing was filed"
        ));
    }
    Ok(Claimed {
        workspace,
        name: text("name")?,
        grade: text("grade")?,
        addresses,
        ca: text("ca")?,
        cert: text("cert")?,
    })
}

fn missing(key: &str) -> String {
    format!("the server's answer carried no `{key}`; nothing was filed")
}

/// **File the entry**, with `key` as its private key, and answer where.
pub fn file(data_root: &Path, claimed: &Claimed, key: &str) -> Result<PathBuf, String> {
    let entries = entries::dir(data_root);
    let dir = entries.join(&claimed.workspace);
    std::fs::create_dir_all(&entries).map_err(|e| format!("{}: {e}", entries.display()))?;
    std::fs::create_dir(&dir).map_err(|e| {
        if e.kind() == std::io::ErrorKind::AlreadyExists {
            format!(
                "this box already holds an entry for workspace {:?} at {} and it was not \
                 overwritten. The offer is spent and its certificate was not kept: move or \
                 remove that directory, have the offer declined, and register again with a \
                 new one",
                claimed.workspace,
                dir.display()
            )
        } else {
            format!("{}: {e}", dir.display())
        }
    })?;
    let address = claimed.addresses.join("\n");
    for (leaf, body) in [
        (ANCHORS, claimed.ca.as_str()),
        (CHAIN, claimed.cert.as_str()),
        (KEY, key),
        (ADDRESS, address.as_str()),
    ] {
        laid(&dir.join(leaf), body)?;
    }
    Ok(dir)
}

/// One file, created readable by this account alone — the mode rides the
/// `open`, so the private key is never wider even for an instant.
fn laid(path: &Path, body: &str) -> Result<(), String> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    options
        .open(path)
        .and_then(|mut file| {
            std::io::Write::write_all(&mut file, format!("{}\n", body.trim_end()).as_bytes())
        })
        .map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests;
