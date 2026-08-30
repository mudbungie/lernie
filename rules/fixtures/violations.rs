//! DELIBERATE ast-grep fixture — NOT part of the crate and never compiled. It
//! lives under `rules/`, outside `src/`, and is named by no Cargo target. Its
//! only job is to be flagged by every rule in `rules/`.
//!
//! Smoke test, BOTH DIRECTIONS (see the `rules-audit` Makefile target):
//!   - `ast-grep scan src` MUST exit zero.
//!   - every rule in `rules/`, **run alone by its own id**, must flag
//!     something here:
//!       * no-rc-refcell.yml            → violations 1–2
//!       * no-pub-borrow-return.yml     → violations 3–4
//!       * no-pub-generic-bounds.yml    → violation 5
//!       * no-named-lifetimes.yml       → violation 6
//!       * no-assert-outside-tests.yml  → violation 7
//!       * no-lint-suppression.yml      → violation 8
//!       * unsafe-outside-sys.yml       → violations 9–10
//!       * locks-outside-state.yml      → violation 11
//!       * no-bare-command.yml          → violation 12
//!       * no-bare-fork.yml             → violation 13
//!
//! One direction alone is worthless. A clean `src` proves nothing if a rule's
//! pattern has silently stopped matching anything at all — which is exactly
//! how a gate passes as green forever. If any violation below ever stops being
//! flagged, that rule has regressed.
//!
//! PER RULE, NOT PER DIRECTORY (bl-1827). The audit used to require only that
//! *something* here was flagged, which the nine surviving rules would satisfy
//! for the tenth forever. It now runs each rule alone, by the `id` read out of
//! its own file, and fails the one that flags nothing — so a rule cannot hide
//! behind its neighbours, and a rule added with no fixture fails on arrival.
//!
//! The four CONFINEMENT rules are the ones with little or nothing to match in
//! `src`: lernie holds no `unsafe` and no lock at all, and its one child
//! process sits in the file its own rule names, so `ast-grep scan src` is
//! silent about all four whether they work or not. That is exactly the state
//! in which a rule passes as green forever, and it is why they are here: for
//! these four, this file is the only thing proving they still work.

// Violation 1: an `Rc` — banned everywhere, no test carve-out.
fn uses_rc() {
    let _r: std::rc::Rc<u32> = std::rc::Rc::new(0);
}

// Violation 2: a `RefCell` — banned everywhere.
struct HoldsRefCell {
    inner: std::cell::RefCell<u32>,
}

// Violation 3: a `pub fn` returning a borrow (the `reference_type` in
// `return_type`). The elided lifetime is the hidden coupling this bans: the
// caller is now tied to the callee's storage without a signature saying so.
pub fn borrow_return(s: &str) -> &str {
    s
}

// Violation 4: a `pub fn` returning an opaque `impl Trait` (the
// `abstract_type` in `return_type`). Under edition 2024's implicit capture an
// `impl Trait` return smuggles borrows invisibly.
pub fn opaque_return() -> impl Iterator<Item = u32> {
    std::iter::empty()
}

// Violation 5: a `pub` item carrying a generic bound (the `type_parameter`
// with a `trait_bounds` child). An UNbounded `<T>` would be clear; the `: Ord`
// is what fires, because a bound on the public surface forces monomorphization
// onto every consumer.
pub struct PubBound<T: Ord> {
    pub first: T,
}

// Violation 6: a named lifetime (the `lifetime` node `'a`; the rule's `not`
// excludes only `'static` and `'_`, which name nothing). Borrow on the way in,
// elided; hand back owned on the way out — then no signature ever needs to
// name one.
pub struct Held<'a> {
    r: &'a str,
}

// Violation 7: an `assert!` outside any test (a `macro_invocation` whose
// `macro` field is `assert`, not inside a `#[cfg(test)]` mod).
fn asserts_in_prod(x: u32) {
    assert!(x > 0, "prod should never assert");
}

// Violation 8: a lint suppression outside tests (the `attribute_item` matching
// `allow(`). Policy lives in Cargo.toml `[lints]`, paired with a
// justification; prod code carries no inline `#[allow]`.
#[allow(clippy::needless_return)]
fn suppresses_a_lint() -> u32 {
    return 0;
}

// Violation 9: an `unsafe` block (the `unsafe_block` node) outside the crate's
// one raw-effect file. The confinement is a LOCATION: the block moves to
// `src/sys.rs`, and the rule's `ignores` list is never widened to meet it.
fn raw_effect_in_the_wrong_file() {
    unsafe {
        let _null: *const u32 = std::ptr::null();
    }
}

// Violation 10: an `unsafe fn` (the `unsafe` keyword inside
// `function_modifiers`). The second physical shape of the same rule — a
// declaration rather than a block — and it is here because a rule that caught
// only one of them would look alive while missing half its subject.
unsafe fn declares_itself_unsafe() {}

// Violation 11: a `Mutex` outside the lock chokepoint. `Arc` beside it is NOT
// matched and is not meant to be: a refcount is not shared mutable state.
struct HoldsALock {
    inner: std::sync::Arc<std::sync::Mutex<u32>>,
}

// Violation 12: a bare `Command::new` (a `call_expression` whose function path
// ends in `Command::new`) outside the crate's one spawn site. lernie forks
// nothing in production; the site the rule names is the suite's own
// out-of-channel certificate mint, and a second one is a design change.
fn builds_a_child_by_hand() {
    let _c = std::process::Command::new("tool");
}

// Violation 13: a bare fork (a zero-argument `.spawn()` / `.output()` /
// `.status()` / `.exec()` method call). Separate from violation 12 because
// building in one file and forking anywhere leaves the ETXTBSY contract open
// — and the fork is the party that holds it.
fn forks_a_child_by_hand(mut cmd: std::process::Command) {
    let _ = cmd.spawn();
}
