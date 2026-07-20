//! `lernie prime` — found the installation substrate idempotently (ARCH
//! §2.2): resolve the harness root and seed the default `models.yaml`,
//! the tool/skill pools, and the `workflows/`/`workspaces/` dirs,
//! seed-if-absent. `make install` runs it.

use super::{Error, Fx, Outcome};
use crate::harness_root;

/// `lernie prime` — takes no arguments.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Seed the harness root — product-less on success (§3.4). Failures —
/// root resolution or seeding — carry the `prime` prefix through one
/// conversion.
pub fn run(_args: Args, _fx: &mut Fx) -> Result<Outcome, Error> {
    go().map_err(|e| Error::new("prime", e))
}

fn go() -> Result<Outcome, Box<dyn std::error::Error>> {
    let roots = harness_root::resolve()?;
    crate::install::prime(&roots)?;
    Ok(Outcome::Quiet)
}
