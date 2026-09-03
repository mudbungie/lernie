//! **The window's own reads** — the two ops whose subject is neither a wall
//! nor a conversation but every channel this box holds (DESIGN §4.21; bl-40ec).
//!
//! A fifth rows file, on the seam [`super::rows`]'s doc already draws four
//! times. `workspaces` and `attention` are the other two members of the same
//! family and each already lives with its own surface; these two had no
//! surface at all until the commands pane and the find pane landed
//! (`crate::ui::commands`, `crate::ui::find`).
//!
//! **Neither names a workspace**, so neither can be routed to one channel:
//! its subject is all of them, read off its own empty or workspace-less
//! `params` and nowhere else ([`super::Verb::addresses_a_workspace`]). The
//! CLI has fanned on that predicate since bl-0d54 (`crate::cli::Decided::
//! Fanned`, `crate::seat::fan`) and the window's poster does now
//! (`crate::offframe::poster`), so all three surfaces read one rule.
//!
//! # One of the two has an argv row and the other cannot
//!
//! [`SEARCH`] is a row: one string parameter, spellable, and `lernie search
//! <text>` fans over every channel exactly as `lernie workspaces` does.
//!
//! [`HELP`] is **not**, and the reason is that its word is already taken by a
//! different question. `lernie help` answers *what does this BINARY take*,
//! from a table compiled into it, with nothing provisioned and no engine up
//! (`super::help`); the wire's `help` answers *what does that ENGINE offer*.
//! Two subjects, one word, and argv has one namespace — so the engine's is
//! reached from the window and `lernie ask '{"op":"help"}'` stays the escape
//! hatch argv already has for every op with no row. It is the roster-without-a-
//! row shape [`super`] already documents for `prepare`, `prompt`, `effort` and
//! `priority`, taken for a fourth reason.

use serde_json::Value;

use super::Verb;

/// **The engine's own verb table.** No row: see the module doc.
pub const HELP: Verb = Verb {
    word: "help",
    params: &[],
    summary: "every op this engine has a word for, and what each is for",
    detail: "One row per op the engine answers: the line to type, one \
             sentence on what it is for, the page under that, and whether the \
             op is spoken by an operator or by a program. It takes no address, \
             so its subject is EVERY channel this box holds — two engines may \
             be at two protocol versions, and a union would say they are one \
             thing. It is also the table every seat's interface parity is \
             judged against, so the pane an operator reads and the ledger that \
             reddens for a missing control come off one answer.",
};

/// **Text found across everything one engine can see.**
pub const SEARCH: Verb = Verb {
    word: "search",
    params: &["text"],
    summary: "find text across balls, workspaces and conversations",
    detail: "One row per hit: what kind of thing carried it, which field of \
             that thing, how far into it, and the words around it — plus what \
             the engine could not read, which is a different claim from \
             finding nothing there. It takes no address, so its subject is \
             EVERY channel this box holds and the answer is printed under the \
             name of the channel each came down. A hit is READ and not \
             actionable: its workspace and project are the engine's own \
             absolute paths rather than the names every gesture carries, so \
             feeding one back earns `unknown workspace` (yog bl-ef16).",
};

/// The engine's table, typed — a door whose arity is its signature, on the
/// same terms as every other row's.
pub fn help() -> Value {
    HELP.built(Vec::new())
}

/// The search, typed.
pub fn search(text: String) -> Value {
    SEARCH.built(vec![text])
}
