//! Generate JSON Schemas for the v0.3 conversation-repo + harness-root
//! config files.
//!
//! Usage: `cargo run --bin gen-schemas -- <out-dir>`
//! (or via `make schemas`).

use lernie::config::schemas;
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let out = match args.next() {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from("schemas"),
    };
    match schemas::write_to(&out) {
        Ok(written) => {
            for path in &written {
                println!("wrote {}", path.display());
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("gen-schemas: {e}");
            ExitCode::FAILURE
        }
    }
}
