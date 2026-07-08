//! Inference-provider runtime types.
//!
//! Per `docs/ARCHITECTURE.md` §4.1, a provider is an (endpoint, auth) pair.
//! The concrete HTTP clients and the adapter-contract implementations live
//! in their own workspace crates (e.g. `crates/lernie-provider-anthropic`)
//! so that each provider compiles independently — per
//! `docs/PRINCIPLES.md` "Integrations are external binaries".
//!
//! [`wire`] pins the harness-side types that parse adapter stdout per
//! the §4.4 response shape, held here (not in a provider crate) so the
//! `lernie` crate carries no library dependency on any specific
//! provider implementation. Config-file parsing for the provider
//! concept lives in [`crate::config::providers`].
//!
//! [`segment`] classifies a closed `response.json`'s last attempt
//! segment (§4.4) across both the legacy v0.3 and the brazen `v=1`
//! event vocabularies — the single seam every framing reader shares
//! through the v0.6 transition.

pub mod segment;
pub mod wire;
