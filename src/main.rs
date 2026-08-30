//! The process entry, and nothing else.
//!
//! It reads argv, hands it to [`lernie::cli::run`], folds in this process's own
//! environment where the decision needs it, writes the verdict's text to the
//! stream the verdict names, and exits with its code. There is no decision
//! here — every one of them is in the library, where a test reads it back as a
//! value. That is what earns this file its place as the single exclusion in
//! `tarpaulin.toml`.
//!
//! The one thing that is the entry point's rather than the library's is
//! **this process's own environment**, folded once into the data root. Every
//! function below this line takes the root as an argument, which is why the
//! suite never needs a process to test one.

use std::process::ExitCode;

use lernie::cli::{Decided, Stream, Verdict};

fn main() -> ExitCode {
    let verdict = match lernie::cli::run(std::env::args().skip(1).collect()) {
        Decided::Say(verdict) => verdict,
        Decided::Entries => rooted(lernie::seat::listing),
        Decided::Ask(envelope) => rooted(|root| lernie::seat::ask(root, &envelope)),
    };
    match verdict.stream {
        Stream::Out => println!("{}", verdict.text),
        Stream::Err => eprintln!("{}", verdict.text),
    }
    ExitCode::from(verdict.code)
}

/// Fold this process's environment into a data root and hand it over, or answer
/// the refusal that says the root is not named.
fn rooted(act: impl FnOnce(&std::path::Path) -> Verdict) -> Verdict {
    match lernie::paths::data_root() {
        Ok(root) => act(&root),
        Err(reason) => Verdict::failed(reason),
    }
}
