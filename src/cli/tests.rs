//! Every decision one invocation can make, read back as a value.
//!
//! Split at the design-time budget along the seam the module itself has:
//! [`verdicts`] is what a run *says* about this binary and how a [`Verdict`]
//! carries it, [`decisions`] is what `run` *decides* to do. The three helpers
//! both halves share stay here.

use super::{Decided, Verdict, run};
use serde_json::Value;

mod decisions;
mod verdicts;

/// Build the argument vector the way `main` does, from string literals.
fn argv(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| (*w).to_string()).collect()
}

/// What a run said, for the arguments that decide to say something.
fn said(words: &[&str]) -> Verdict {
    match run(argv(words)) {
        Decided::Say(verdict) => verdict,
        Decided::Entries => panic!("{words:?} decided to list entries"),
        Decided::Ask(_) => panic!("{words:?} decided to ask"),
    }
}

/// The envelope a run decided to send, for the arguments that decide to ask.
fn asked(words: &[&str]) -> Value {
    match run(argv(words)) {
        Decided::Ask(envelope) => envelope,
        Decided::Entries => panic!("{words:?} decided to list entries"),
        Decided::Say(verdict) => panic!("{words:?} said {:?}", verdict.text),
    }
}
