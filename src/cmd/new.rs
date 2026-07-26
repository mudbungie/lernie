//! `lernie new` — create a workspace and author its first config commit
//! (ARCH §2.2). The descriptions-always snapshot (§3.3) means the
//! data-root pools are resolved at creation, so `roots` is always needed.
//!
//! Because the pools are an *input* to the first config commit, `new`
//! **founds the harness root first**, through the very routine
//! [`lernie prime`](crate::install::prime) runs (§2.2) — not a copy of
//! it. `prime` is seed-if-absent throughout and therefore idempotent, so
//! founding here is a no-op on a primed install and needs no flag: the
//! unseeded data root stops being a special case, and `new` can no
//! longer author a config commit with an empty `descriptions/**` (which
//! would hand every agent forked off it a toolless context, §3.3
//! descriptions-always). Seeding stays single-sourced in `prime`.

use super::{Error, Fx, Outcome, path_line};
use crate::harness_root;
use crate::prompt::{IdGen, NanoIdGen};
use crate::template::{self, RealGit};
use std::path::PathBuf;

/// `lernie new [<path>]`.
#[derive(clap::Args, Debug)]
pub struct Args {
    pub path: Option<PathBuf>,
}

/// Found the harness root, scaffold at `path` (or
/// `<data-root>/workspaces/<auto-id>/`), and print the destination —
/// the verb's one product (§3.4). All failures — root resolution,
/// founding, or scaffolding — carry the `new` prefix through the one
/// conversion point ([`Error::new`]).
pub fn run(args: Args, _fx: &mut Fx) -> Result<Outcome, Error> {
    go(args).map_err(|e| Error::new("new", e))
}

fn go(args: Args) -> Result<Outcome, Box<dyn std::error::Error>> {
    let roots = harness_root::resolve()?;
    // The pools this workspace's `descriptions/**` snapshots from must
    // exist before the snapshot runs (§3.3); founding is seed-if-absent,
    // so a primed install is untouched (§2.2).
    crate::install::prime(&roots)?;
    let dest = args
        .path
        .unwrap_or_else(|| roots.data.join("workspaces").join(NanoIdGen.short()));
    template::scaffold(&dest, &roots, &RealGit::new())?;
    Ok(path_line(dest))
}
