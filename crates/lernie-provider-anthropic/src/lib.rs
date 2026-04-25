//! Reference implementation of the provider-adapter contract
//! (`docs/ARCHITECTURE.md` §4.4) for Anthropic's Messages API.
//!
//! Two layers live here:
//!
//! - [`client`] — a blocking HTTP client for `POST /v1/messages` with the
//!   error taxonomy the adapter needs to classify retry intent. Also
//!   exposes streaming over Anthropic's native SSE wire via
//!   [`client::Client::send_streaming`] + [`streaming::accumulate`].
//! - [`adapter`] — the stdio contract: `run_describe` writes the adapter's
//!   self-description, `run_complete` reads one Messages-API request on
//!   stdin and writes either the parsed response or an in-band error
//!   object on stdout.
//!
//! The binary in `src/main.rs` is a thin shell over [`adapter`] so the
//! logic stays testable without a subprocess dance.
//!
//! This crate is deliberately free of any dependency on the root `lernie`
//! crate: per `docs/PRINCIPLES.md` "Integrations are external binaries",
//! a provider adapter must be publishable out-of-tree without patching
//! the harness. Keep it that way.

pub mod adapter;
pub mod client;

pub use adapter::{
    ADAPTER_NAME, AUTH_ENV, CAPABILITIES, DEFAULT_ENDPOINT, ENDPOINT_ENV, MODELS, SCHEMA_VERSION,
    run_complete, run_describe,
};
pub use client::streaming::{self, Event, EventStream, accumulate};
pub use client::{
    ANTHROPIC_VERSION, Client, ContentBlock, Error, Message, Request, Response, Role, Usage,
};
