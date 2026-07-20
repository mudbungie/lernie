//! `lernie config` — author a config commit beyond `lernie new` (ARCH
//! §2.2, §2.3): the only act besides `new` that advances a config
//! branch. The interactive `$EDITOR` hand-off arrives through
//! [`Fx::editor`](super::Fx::editor); everything else lives in
//! [`crate::template::authoring::from_cli`].

use super::{Error, Fx, Outcome};
use crate::harness_root;
use crate::template::{self, RealGit};
use std::path::PathBuf;

/// `lernie config <workspace> [<name>] [--from <source>] [--orphan]`.
#[derive(clap::Args, Debug)]
pub struct Args {
    pub workspace: PathBuf,
    pub name: Option<String>,
    /// Fork a new branch off `config/<source>` instead of advancing.
    #[arg(long)]
    pub from: Option<String>,
    /// Start a fresh orphan lineage instead of advancing.
    #[arg(long)]
    pub orphan: bool,
}

/// Materialize, edit via [`Fx::editor`](super::Fx::editor), and commit —
/// product-less on success (§3.4). Failures — root resolution or the
/// authoring pass — carry the `config` prefix through one conversion.
pub fn run(args: Args, fx: &mut Fx) -> Result<Outcome, Error> {
    go(args, fx).map_err(|e| Error::new("config", e))
}

fn go(args: Args, fx: &mut Fx) -> Result<Outcome, Box<dyn std::error::Error>> {
    let roots = harness_root::resolve()?;
    template::authoring::from_cli(
        &args.workspace,
        &roots.data,
        args.name.as_deref(),
        args.from.as_deref(),
        args.orphan,
        fx.editor,
        &RealGit::new(),
    )?;
    Ok(Outcome::Quiet)
}
