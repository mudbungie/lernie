//! Filesystem watcher for a conversation repo.
//!
//! Exposes the subset of repo paths listed in ARCHITECTURE.md §3.5 as a
//! drainable stream of coalesced change notifications. The module is pure
//! Rust — no egui/eframe dependency — so a future `lernie-ui-web` crate can
//! reuse it unchanged. The watcher is strictly read-only: it never mutates
//! the repo.
//!
//! Backing impl: `notify::RecommendedWatcher` (inotify on Linux, kqueue on
//! BSD/macOS, polling fallback elsewhere). Coalescing collapses multiple
//! events for the same path within one tick window — rapid sequential
//! writes and atomic-rename sequences both emerge as a single change per
//! destination path.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};

use notify::{
    Event, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher,
    event::{EventKind, ModifyKind, RenameMode},
};

/// Repo-relative paths (or prefixes) the UI watches, per ARCH §3.5.
/// Branch existence is read from `.git/refs/` — no sidecar state
/// file (PRINCIPLES.md "Single source of truth").
const WATCHED_PREFIXES: &[&str] = &[
    ".agent/goal.md",
    ".agent/compactions",
    "exchanges",
    "invocations",
    "artifacts",
    "tools",
    ".git/HEAD",
    ".git/refs",
];

#[derive(Debug, thiserror::Error)]
#[error("filesystem watcher: {0}")]
pub struct WatchError(#[from] notify::Error);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Touched,
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub path: PathBuf,
    pub kind: ChangeKind,
}

pub struct Watcher {
    repo_root: PathBuf,
    _inner: RecommendedWatcher,
    rx: Receiver<notify::Result<Event>>,
}

impl Watcher {
    pub fn new(repo_root: &Path) -> Result<Self, WatchError> {
        let (tx, rx) = mpsc::channel();
        let mut inner: RecommendedWatcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })?;
        inner.watch(repo_root, RecursiveMode::Recursive)?;
        Ok(Self {
            repo_root: repo_root.to_path_buf(),
            _inner: inner,
            rx,
        })
    }

    /// Drain pending notify events and return one coalesced `Change` per
    /// affected watched path. Paths outside the §3.5 allowlist are dropped;
    /// rename-source events are dropped in favor of the destination.
    pub fn tick(&self) -> Vec<Change> {
        let mut raw: Vec<(PathBuf, EventKind)> = Vec::new();
        loop {
            match self.rx.try_recv() {
                Ok(Ok(event)) => ingest(event, &mut raw),
                Ok(Err(_)) => {}
                Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
            }
        }
        coalesce(&self.repo_root, raw)
    }
}

fn ingest(event: Event, raw: &mut Vec<(PathBuf, EventKind)>) {
    let paths = event.paths;
    match event.kind {
        EventKind::Modify(ModifyKind::Name(RenameMode::Both)) if paths.len() == 2 => {
            let mut iter = paths.into_iter();
            let from = iter.next().unwrap();
            let to = iter.next().unwrap();
            raw.push((from, EventKind::Modify(ModifyKind::Name(RenameMode::From))));
            raw.push((to, EventKind::Modify(ModifyKind::Name(RenameMode::To))));
        }
        kind => {
            for path in paths {
                raw.push((path, kind));
            }
        }
    }
}

fn coalesce(repo_root: &Path, raw: Vec<(PathBuf, EventKind)>) -> Vec<Change> {
    let mut latest: HashMap<PathBuf, EventKind> = HashMap::new();
    let mut order: Vec<PathBuf> = Vec::new();
    for (path, kind) in raw {
        if !is_watched(repo_root, &path) {
            continue;
        }
        if matches!(kind, EventKind::Modify(ModifyKind::Name(RenameMode::From))) {
            if latest.remove(&path).is_some() {
                order.retain(|p| p != &path);
            }
            continue;
        }
        if !latest.contains_key(&path) {
            order.push(path.clone());
        }
        latest.insert(path, kind);
    }
    order
        .into_iter()
        .map(|p| {
            let kind = latest[&p];
            let change_kind = classify(kind, &p);
            Change {
                path: p,
                kind: change_kind,
            }
        })
        .collect()
}

fn classify(kind: EventKind, path: &Path) -> ChangeKind {
    match kind {
        EventKind::Remove(_) => ChangeKind::Removed,
        _ => {
            if path.exists() {
                ChangeKind::Touched
            } else {
                ChangeKind::Removed
            }
        }
    }
}

fn is_watched(repo_root: &Path, path: &Path) -> bool {
    let Ok(rel) = path.strip_prefix(repo_root) else {
        return false;
    };
    let rel_str = rel.to_string_lossy();
    WATCHED_PREFIXES
        .iter()
        .any(|prefix| rel_str == *prefix || rel_str.starts_with(&format!("{}/", prefix)))
}

#[cfg(test)]
mod tests;
