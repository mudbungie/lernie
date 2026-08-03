//! The `lernie-eval-agent` binary — thin wiring over the library
//! (`lib.rs`), where all logic and its coverage live. Reads the README
//! "Run the suite" driver contract from argv/env and drives `lernie`
//! (from `PATH`) through one evaluation run. Its exit code is ignored
//! by the runner by contract; failures still land on stderr.
//!
//! `--version` as argv\[1\] is the one non-run invocation (bl-36fa):
//! the runner probes it once per evaluation and records the line among
//! the reproducibility inputs, so it must answer before the contract's
//! env checks — a version probe carries no `LERNIE_HOME`.

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    if let Some(line) = lernie_eval_agent::version_answer(env::args().nth(1).as_deref()) {
        println!("{line}");
        return ExitCode::SUCCESS;
    }
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("lernie-eval-agent: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let contract = lernie_eval_agent::Contract::assemble(
        env::args().nth(1),
        env::var_os("LERNIE_HOME").map(Into::into),
        env::var_os("LERNIE_EXPERIMENT").map(Into::into),
        env::var_os("LERNIE_EVAL_REPORT").map(Into::into),
        env::current_dir()?,
    )?;
    let machine_config = lernie_eval_agent::machine_config_root(
        env::var_os("XDG_CONFIG_HOME").as_deref(),
        env::var_os("HOME").as_deref(),
    );
    lernie_eval_agent::drive("lernie".as_ref(), machine_config.as_deref(), &contract)?;
    Ok(())
}
