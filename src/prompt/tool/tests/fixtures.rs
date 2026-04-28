//! Shared test fixtures. Helpers lay down executable shell scripts in
//! a tempdir-rooted harness root so [`super::super::SpawnTool`] can
//! resolve them via the §3.3 lookup order without touching `PATH`.

use crate::prompt::Clock;
use std::cell::RefCell;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Deterministic [`Clock`] — `started_at` / `ended_at` come back as
/// `iso-1` and `iso-2` so the on-disk record's timestamps are
/// observable in assertions without dragging the wall clock in.
#[derive(Default)]
pub(super) struct FixedClock {
    iso_calls: RefCell<u32>,
}

impl Clock for FixedClock {
    fn now_iso8601(&self) -> String {
        *self.iso_calls.borrow_mut() += 1;
        format!("iso-{}", self.iso_calls.borrow())
    }
    fn now_compact(&self) -> String {
        // Unused by the executor but the trait demands it.
        "ct".into()
    }
}

/// Harness root containing a `tools/` subdir; the [`TempDir`] is held
/// so its lifetime spans the whole test. Tests interact with it
/// through [`Self::install`] which drops a script into
/// `tools/lernie-tool-<name>`.
pub(super) struct HarnessRoot {
    pub(super) dir: TempDir,
}

impl HarnessRoot {
    pub(super) fn new() -> Self {
        let dir = TempDir::new().expect("tempdir");
        std::fs::create_dir_all(dir.path().join(super::super::TOOLS_DIR)).expect("mkdir tools/");
        Self { dir }
    }

    pub(super) fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Drop a chmod-+x shell script under
    /// `<root>/tools/lernie-tool-<name>` and return its absolute path.
    pub(super) fn install(&self, name: &str, body: &str) -> PathBuf {
        let path = self.dir.path().join(super::super::TOOLS_DIR).join(format!(
            "{}{}",
            super::super::EXTERNAL_PREFIX,
            name
        ));
        write_script(&path, body);
        path
    }
}

/// Write `body` to `path`, prepend the bash shebang, and chmod 0o755
/// so the kernel will exec it. Used both by the harness-root installer
/// and by tests that need a binary outside the harness root.
pub(super) fn write_script(path: &Path, body: &str) {
    let mut script = String::from("#!/usr/bin/env bash\n");
    script.push_str(body);
    if !script.ends_with('\n') {
        script.push('\n');
    }
    std::fs::write(path, script).expect("write script");
    let mut perm = std::fs::metadata(path).expect("stat").permissions();
    perm.set_mode(0o755);
    std::fs::set_permissions(path, perm).expect("chmod");
}

/// Per-test step directory. Mirrors the v0.3.1 layout
/// `<conv-repo>/steps/<conv-id>/<NNN>/` (ARCH §2.2 / §2.3 — at the
/// conv-repo root, outside every worktree) so the executor lands
/// `tools/<tool-id>/` underneath. `_root` is held only for its
/// `Drop` — the tempdir cleanup happens when [`StepDir`] goes out
/// of scope.
pub(super) struct StepDir {
    _root: TempDir,
    pub(super) path: PathBuf,
}

impl StepDir {
    pub(super) fn new() -> Self {
        let root = TempDir::new().expect("step tempdir");
        let path = root.path().join("steps").join("convid").join("001");
        std::fs::create_dir_all(&path).expect("mkdir step");
        Self { _root: root, path }
    }
}
