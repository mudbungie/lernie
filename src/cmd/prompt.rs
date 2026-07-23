//! `lernie prompt` — drive one root conversation (ARCH §2.3). The §2.9
//! preludes (`become_pgid_leader` + `install_stop_handler`) are the
//! binding's, run before [`run`] (ARCH §3.4 binding-preludes seam,
//! [`super::prelude`]); this entry only builds the deps and drives.

use super::{Error, Fx, Outcome};
use crate::harness_root;
use crate::prompt::inbox::AdvanceLauncher;
use crate::prompt::{self, NanoIdGen, SpawnAdapter, SpawnTool, SystemClock};
use crate::template::RealGit;
use std::path::PathBuf;

/// `lernie prompt <repo> <message>`.
#[derive(clap::Args, Debug)]
pub struct Args {
    pub repo: PathBuf,
    pub message: String,
}

/// Spawn the root agent branch and drive its step loop; print the agent
/// id (§2.3) — the verb's one product. The `String`→[`Outcome::Line`]
/// map and the failure conversion are fn-pointers, so the success arm
/// carries no test-only region (its happy path needs a live provider,
/// pinned by `tests/prompt_end_to_end.rs`).
pub fn run(args: Args, fx: &mut Fx) -> Result<Outcome, Error> {
    go(args, fx)
        .map(Outcome::Line)
        .map_err(|e| Error::new("prompt", e))
}

/// No workspace scan: drivers touch only their own branch (§2.11). The
/// detached-launch and successor target is
/// [`Fx::driver_target`](super::Fx::driver_target); the stop flag is
/// [`Fx::stop`](super::Fx::stop).
fn go(args: Args, fx: &mut Fx) -> Result<String, Box<dyn std::error::Error>> {
    let roots = harness_root::resolve()?;
    // The binding-injected driver target (§2.11 "injected at the binding,
    // not resolved by name") serves both re-entry seams: the §3.3 tool
    // resolver's third hop and the §2.11/§6 detached launch. No
    // `current_exe` here — under a linked host it would name the host.
    let tool_executor = SpawnTool::new(&roots.data, &SystemClock, &fx.driver_target);
    let launcher = AdvanceLauncher::with_exe(fx.driver_target.clone());
    let deps = prompt::Deps {
        adapter: &SpawnAdapter,
        sleeper: &prompt::RealSleeper,
        git: &RealGit::new(),
        clock: &SystemClock,
        id_gen: &NanoIdGen,
        tool_executor: &tool_executor,
        config_root: &roots.config,
        adapter_target: fx.adapter_target.as_deref(),
        stop: fx.stop,
        launcher: &launcher,
    };
    prompt::run(&args.repo, &args.message, &deps).map_err(Into::into)
}
