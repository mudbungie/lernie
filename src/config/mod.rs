//! Configuration files for a conversation repo.
//!
//! Each submodule owns one file under `.agent/` per ARCH §2.2 — see
//! [`version`], [`providers`], [`agents`], [`manifest`], [`workflow`].
//! [`cross`] enforces references across files (e.g. `agents.*.model` must
//! resolve to a defined model).

pub mod action;
pub mod agents;
pub mod cross;
pub mod error;
pub mod manifest;
pub mod providers;
pub mod schemas;
pub mod version;
pub mod workflow;

pub use action::{Action, DispatchMode};
pub use agents::{AgentRole, Agents};
pub use error::{LoadError, Warning};
pub use manifest::{Manifest, OverflowPolicy};
pub use providers::{Auth, Capabilities, Model, Provider, Providers};
pub use version::Version;
pub use workflow::{CompactionTrigger, Event, Workflow};
