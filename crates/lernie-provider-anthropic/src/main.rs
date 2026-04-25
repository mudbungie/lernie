//! `lernie-provider-anthropic` — the reference implementation of the
//! provider-adapter contract (ARCH §4.4) for Anthropic's Messages API.
//!
//! This file is intentionally skeletal: argv parsing, a SIGTERM handler,
//! stdin/stdout wiring. All logic lives in
//! [`lernie_provider_anthropic::adapter`], where it is testable without a
//! subprocess dance.
//!
//! Usage:
//!   lernie-provider-anthropic describe
//!   lernie-provider-anthropic complete [--request <path>]
//!
//! `describe` writes a single JSON object to stdout. `complete` reads one
//! Anthropic Messages-API request and writes one JSON object (either the
//! upstream response or an in-band error object per §4.4) to stdout. The
//! request is read from stdin by default; `--request <path>` reads it from
//! a file instead (semantics identical — additive per ARCH §4.4). The
//! process exits `0` in both cases; non-zero exit is reserved for
//! adapter-side crashes.
//!
//! The upstream endpoint comes from the env var named in
//! `describe.endpoint_env` (currently `LERNIE_PROVIDER_ANTHROPIC_ENDPOINT`),
//! falling back to [`DEFAULT_ENDPOINT`]. ARCH §4.4 reserves endpoint
//! interpretation to the adapter; the harness only forwards the value.

use clap::{Parser, Subcommand};
use lernie_provider_anthropic::adapter::set_stop;
use lernie_provider_anthropic::{DEFAULT_ENDPOINT, ENDPOINT_ENV, run_complete, run_describe};
use std::fs::File;
use std::io::{self, BufReader, Write};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "lernie-provider-anthropic",
    about = "Anthropic provider adapter"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Write the adapter's self-description JSON to stdout.
    Describe,
    /// Read a Messages-API request, write one response or error object on
    /// stdout. Reads from stdin by default; `--request <path>` reads from a
    /// file instead (ARCH §4.4, additive).
    Complete {
        /// Read the request JSON from this file instead of stdin.
        #[arg(long, value_name = "PATH")]
        request: Option<PathBuf>,
    },
}

/// Install a SIGTERM handler that flushes a terminal error event and
/// fast-exits the process.
///
/// Contract (ARCH §4.4): on SIGTERM the adapter must drop any in-flight
/// HTTP request, flush partial state — for streaming, emit a final
/// `message_stop` or `error` event — and exit within 5 seconds.
///
/// `reqwest::blocking` runs the HTTP socket read inside its own tokio
/// runtime which transparently restarts the syscall on `EINTR`, so a
/// flag-only handler cannot wake the main thread. We therefore write
/// the terminal `error` JSON line to stdout from the handler itself
/// (via async-signal-safe `write(2)`) and call `_exit(0)`. The same
/// terminal-error shape is legal in both non-streaming and streaming
/// output (one trailing JSONL event), so a single handler serves both.
/// We also flip the [`set_stop`] flag so [`adapter::streaming::drain`]
/// short-circuits if it happens to observe it before the handler runs
/// to completion (covered by the unit tests).
fn install_sigterm_handler() {
    extern "C" fn on_sigterm(_signo: libc::c_int) {
        // Async-signal-safe primitives only past this point.
        set_stop();
        const MSG: &[u8] = b"{\"type\":\"error\",\"kind\":\"retryable\",\"http_status\":null,\"message\":\"interrupted by SIGTERM\",\"retry_after_seconds\":null}\n";
        // SAFETY: `write(2)` and `_exit(2)` are explicitly listed as
        // async-signal-safe by POSIX. fd 1 is the process's stdout.
        unsafe {
            let _ = libc::write(1, MSG.as_ptr() as *const _, MSG.len());
            libc::_exit(0);
        }
    }
    // SAFETY: `on_sigterm` only calls async-signal-safe functions. The
    // registration itself is the documented way to install a signal
    // handler on POSIX.
    unsafe {
        libc::signal(libc::SIGTERM, on_sigterm as *const () as libc::sighandler_t);
    }
}

fn main() -> ExitCode {
    install_sigterm_handler();

    let cli = Cli::parse();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let result = match cli.command {
        Command::Describe => run_describe(&mut out),
        Command::Complete { request } => {
            let api_key = std::env::var("ANTHROPIC_API_KEY").ok();
            let endpoint =
                std::env::var(ENDPOINT_ENV[0]).unwrap_or_else(|_| DEFAULT_ENDPOINT.to_string());
            match request {
                Some(path) => match File::open(&path) {
                    Ok(file) => {
                        let mut reader = BufReader::new(file);
                        run_complete(&mut reader, &mut out, api_key.as_deref(), &endpoint)
                    }
                    Err(e) => Err(io::Error::new(
                        e.kind(),
                        format!("--request {}: {e}", path.display()),
                    )),
                },
                None => {
                    let stdin = io::stdin();
                    let mut reader = stdin.lock();
                    run_complete(&mut reader, &mut out, api_key.as_deref(), &endpoint)
                }
            }
        }
    };
    match result {
        Ok(()) => {
            let _ = out.flush();
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("lernie-provider-anthropic: {e}");
            ExitCode::FAILURE
        }
    }
}
