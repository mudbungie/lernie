//! `lernie tool <name>` — in-process built-in tool entry (ARCH §3.3):
//! `tool_use.input` JSON on stdin, bytes on stdout, exit 0/non-zero. The
//! stdio arrives through [`Fx`](super::Fx) (locked by the binding), as
//! does the driver target the `dispatch` / `message` built-ins re-enter
//! the front door with (§2.11 — the same injected target the §3.3 tool
//! resolver's third hop addresses).

use super::{Error, Fx, Outcome};
use crate::prompt::tool::builtin;

/// `lernie tool <name>`.
#[derive(clap::Args, Debug)]
pub struct Args {
    pub name: String,
}

/// Delegate to [`builtin::run`] over the injected stdio; the process
/// exit code rides back as [`Outcome::Code`](super::Outcome::Code)
/// (§3.3). The failure prefix is `tool <name>`, as today.
pub fn run(args: Args, fx: &mut Fx) -> Result<Outcome, Error> {
    let code = builtin::run(
        &args.name,
        &fx.driver_target,
        &mut fx.tool_stdin,
        &mut fx.tool_stdout,
        &mut fx.tool_stderr,
    )
    .map_err(|e| Error::new(format!("tool {}", args.name), e))?;
    // Tool exit codes ride within `u8` (POSIX), so `as u8` is faithful.
    Ok(Outcome::Code(code as u8))
}
