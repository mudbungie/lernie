//! Inference-provider runtime clients.
//!
//! Per `docs/ARCHITECTURE.md` §4.1, a provider is an (endpoint, auth) pair.
//! The concrete HTTP clients and the adapter-contract implementations live
//! in their own workspace crates (e.g. `crates/lernie-provider-anthropic`)
//! so that each provider compiles independently — per
//! `docs/PRINCIPLES.md` "Integrations are external binaries". Config-file
//! parsing for the provider concept lives in [`crate::config::providers`].
