//! Inference-provider runtime clients.
//!
//! Per `docs/ARCHITECTURE.md` §4.1, a provider is an (endpoint, auth) pair.
//! The submodules here are the HTTP clients that realize a model call as an
//! API call against a concrete provider: one HTTP request, one parsed
//! response. Config-file parsing for the same concept lives in
//! [`crate::config::providers`].

pub mod anthropic;
pub mod anthropic_adapter;
