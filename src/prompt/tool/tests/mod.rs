//! Tests for the tool executor (ARCH §3.3). Split by axis so each
//! file stays under the 300-line cap.
//!
//! - [`fixtures`]: shared clock + test-side helpers for laying down
//!   fixture tool scripts in a tempdir-rooted harness root.
//! - [`types`]: round-trips and small invariants for the on-disk
//!   record types and helpers in `super::super`.
//! - [`resolve`]: §3.3 resolution order — harness-root, PATH, and the
//!   injected-driver-target third hop. There is no not-found case: the
//!   third hop always resolves (§2.11 injected target).
//! - [`happy`]: end-to-end stdio contract — exit 0, exit non-zero,
//!   stderr concat-on-error, on-disk record shape.
//! - [`cascade`]: SIGTERM-then-SIGKILL semantics, and the "tool died
//!   from a signal not under harness control" §2.10 fault.
//! - [`errors`]: failure modes of resolution, spawn, and disk-record
//!   I/O.
//! - [`bash_tool`], [`read_file_tool`]: end-to-end through the
//!   cargo-built `lernie` binary (the §3.3 third hop), injected as the
//!   driver target via [`crate::test_support::lernie_binary`].

mod bash_tool;
mod cascade;
mod errors;
mod fixtures;
mod happy;
mod read_file_tool;
mod resolve;
mod types;
