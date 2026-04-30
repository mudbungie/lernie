//! Orchestration tests for [`super::run`]. Wires a stub
//! [`super::BranchInspector`], stub [`super::PgidFinder`], and
//! recording [`super::Signaler`] together over real on-disk
//! fixtures so the only pieces under test are the orchestration
//! logic itself: branch validation, step-dir discovery, pgid
//! de-duplication, idempotence.
//!
//! Real signal-cascade integration (kernel-level kills) lives in
//! `tests/stop_cli.rs` so unit tests stay fast and process-isolated.
//!
//! Split by axis so each file stays under the 300-line cap:
//!
//! - [`fixtures`]: stub inspectors / finders / git runner + helpers
//!   for materializing step trees on disk.
//! - [`orchestration`]: the happy-path tests over branch +
//!   step discovery + pgid signalling.
//! - [`edge_cases`]: error propagation, idempotence corner cases,
//!   and small surface coverage (Error Display, NoopGit methods).

mod edge_cases;
mod fixtures;
mod orchestration;
