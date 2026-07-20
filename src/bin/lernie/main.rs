//! The `lernie` harness binary — the **exec binding** of the command
//! surface (ARCH §3.4 "One command surface, two bindings").
//!
//! The verbs, their arguments, and their products are defined once, in
//! [`lernie::cmd`] ([`lernie::cmd::Command`] is the authoritative
//! inventory, §3.4). This binary is the thin exec binding: it parses the
//! shared [`Cli`], runs the per-verb §2.9 preludes ([`prelude`](lernie::cmd::prelude)),
//! builds the [`Fx`] injections (the running-binary path, the `$EDITOR`
//! spawn, the locked stdio, the stop flag), invokes
//! [`Command::run`](lernie::cmd::Command::run), and performs the returned
//! [`Outcome`] — printing a product, exec'ing a successor, or mapping a
//! tool exit code. Every process-global and terminal effect lives here,
//! at the binding, never in the surface.
//!
//! Verb inventory (mirrors [`Command`](lernie::cmd::Command), the one authoritative
//! definition): `new`, `config`, `prompt`, `dispatch`, `stop`,
//! `message`, `scan`, `bundle`, `replay`, `advance`, `tool`, `prime`.

mod cli;

use clap::Parser;
use lernie::cmd::{Cli, Command, Fx, Outcome, prelude};
use std::io;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();
    // The §2.9 binding preludes, run per parsed verb before the surface is
    // invoked (ARCH §3.4 binding-preludes seam) — never inside a verb entry.
    run_preludes(&cli.command);

    // The one `current_exe` of the launch/successor family (§2.11/§3.4):
    // resolved here at the injection point and threaded down as
    // `Fx::driver_target`. Its (exotic) failure exits before any work.
    let driver_target = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("lernie: resolve current executable: {e}");
            return ExitCode::FAILURE;
        }
    };
    let editor: fn(&Path) -> io::Result<()> = cli::edit_in_editor;

    // Stdio is locked for the whole verb (the `tool` verb writes raw
    // bytes into it, §3.3) and released before any product is printed —
    // holding the `StdoutLock` across a `println!` would deadlock.
    let result = {
        let mut stdin = io::stdin().lock();
        let mut stdout = io::stdout().lock();
        let mut stderr = io::stderr().lock();
        let mut fx = Fx {
            driver_target,
            editor: &editor,
            tool_stdin: &mut stdin,
            tool_stdout: &mut stdout,
            tool_stderr: &mut stderr,
            stop: prelude::stop_flag(),
        };
        cli.command.run(&mut fx)
    };

    match result {
        Ok(outcome) => perform(outcome),
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

/// The §2.9 process-group-leadership and stop-flag preludes the binding
/// performs before a driver verb (ARCH §3.4, [`prelude`](lernie::cmd::prelude)): `prompt`
/// and `advance` take a pgid and install the stopped-deposit handler;
/// `dispatch` (child re-entry) takes a pgid. Every other verb needs
/// neither.
fn run_preludes(command: &Command) {
    match command {
        Command::Prompt(_) | Command::Advance(_) => {
            prelude::become_pgid_leader();
            prelude::install_stop_handler();
        }
        Command::Dispatch(_) => prelude::become_pgid_leader(),
        _ => {}
    }
}

/// Perform a verb's [`Outcome`] (ARCH §3.4): print the one product, do
/// nothing, `exec` the successor (a successful `execve` never returns —
/// reaching past it is the failure path), or map the tool exit code.
fn perform(outcome: Outcome) -> ExitCode {
    match outcome {
        Outcome::Line(line) => {
            println!("{line}");
            ExitCode::SUCCESS
        }
        Outcome::Quiet => ExitCode::SUCCESS,
        Outcome::Exec(mut cmd) => {
            use std::os::unix::process::CommandExt;
            eprintln!("lernie advance: exec successor: {}", cmd.exec());
            ExitCode::FAILURE
        }
        // Tool exit codes ride within `u8` (POSIX), preserved by `cmd`.
        Outcome::Code(code) => ExitCode::from(code),
    }
}
