//! `agent-eval` — the experiment × suite × N evaluation runner (ARCH
//! §9.3, v0.10).
//!
//! An **experiment** is a `workflow.yaml` variant under `experiments/`
//! ([`experiment`]); the **suite** is the task set under `tests/suite/`
//! ([`suite`]). The [`runner`] executes experiment × suite × N — seeding
//! an isolated workspace per run, running the task `setup`, invoking the
//! agent through the [`agent`] seam, then running the task `check` (exit
//! 0 the sole pass signal) — and [`stats`] aggregates the outcomes into
//! pass@1 (with 95% Wilson intervals) and pass@5, overall and per
//! category. [`report`] renders the result.
//!
//! The agent invocation is behind a trait ([`agent::Agent`]) so the whole
//! runner is testable without live model traffic; the production
//! implementation shells out to an external harness-driver binary.

pub mod agent;
pub mod experiment;
pub mod report;
pub mod runner;
pub mod stats;
pub mod suite;
