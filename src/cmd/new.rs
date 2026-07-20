//! `lernie new` — create a workspace and author its first config commit
//! (ARCH §2.2). The descriptions-always snapshot (§3.3) means the
//! data-root pools are resolved at creation, so `roots` is always needed.

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

/// Scaffold at `path` (or `<data-root>/workspaces/<auto-id>/`) and print
/// the destination — the verb's one product (§3.4). All failures — root
/// resolution or scaffolding — carry the `new` prefix through the one
/// conversion point ([`Error::new`]).
pub fn run(args: Args, _fx: &mut Fx) -> Result<Outcome, Error> {
    go(args).map_err(|e| Error::new("new", e))
}

fn go(args: Args) -> Result<Outcome, Box<dyn std::error::Error>> {
    let roots = harness_root::resolve()?;
    let dest = args
        .path
        .unwrap_or_else(|| roots.data.join("workspaces").join(NanoIdGen.short()));
    template::scaffold(&dest, &roots.data, &RealGit::new())?;
    Ok(path_line(dest))
}
