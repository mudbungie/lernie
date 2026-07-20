//! The binding preludes (ARCH §3.4 binding-preludes seam).
//!
//! These are the process-group-leadership and stop-flag mechanisms
//! (§2.9) that **the binding**, not a verb entry, performs identically
//! before invoking a driver verb: promote to a process-group leader, so
//! the §2.9 cascade reaches this executor's own adapter and tool
//! subprocesses without escaping to the shell/UI; install the SIGTERM
//! handler that flips [`stop_flag`]; and hand that flag to the driver
//! verbs through [`super::Fx::stop`].
//!
//! The surface *supplies* them here; the binding *invokes* them (the
//! exec binding does so per-verb at top-of-`main`, matching the parsed
//! [`super::Command`] — `Prompt`: leader + handler; `Dispatch`: leader;
//! `Advance`: leader + handler). They are never invoked by a verb entry
//! ([`super::Command::run`]) — that is what keeps the command surface
//! free of process-global effect (§3.4 "Process effects stay at the
//! binding"). They are re-exported (not owned) so the mechanisms keep
//! their single home in `prompt::` (`docs/PRINCIPLES.md`, single source
//! of truth).

pub use crate::prompt::stop::become_pgid_leader;
pub use crate::prompt::{install_stop_handler, stop_flag};
