//! Tests for the tool executor (ARCH §3.3). Split by axis so each
//! file stays under the 300-line cap.
//!
//! - [`fixtures`]: shared clock + test-side helpers for laying down
//!   fixture tool scripts in a tempdir-rooted harness root.
//! - [`types`]: round-trips and small invariants for the on-disk
//!   record types and helpers in `super::super`.
//! - [`resolve`]: §3.3 resolution order — harness-root, PATH,
//!   in-process fallback, and the not-found terminal case.
//! - [`happy`]: end-to-end stdio contract — exit 0, exit non-zero,
//!   stderr concat-on-error, on-disk record shape.
//! - [`cascade`]: SIGTERM-then-SIGKILL semantics, and the "tool died
//!   from a signal not under harness control" §2.10 fault.
//! - [`errors`]: failure modes of resolution, spawn, and disk-record
//!   I/O.

mod cascade;
mod errors;
mod fixtures;
mod happy;
mod resolve;
mod types;
