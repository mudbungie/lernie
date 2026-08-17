//! The **mint seam** — the one deliberate exception to "the public API
//! is the command surface" (ARCH §3.4; the yog bl-aca4 ruling).
//!
//! The agent-name mint's single home is this crate, beside the
//! uniqueness check it races (`workspace::agent_name`): every creation
//! path mints a two-word PascalCase name on omission (`PeachHollow`,
//! ARCH §2.3, bl-79a2). Yog — the linked
//! consumer — must *preview* the very name a fire would mint, so it
//! draws the same function through this facade rather than growing a
//! second list that would drift. Exported: the pure [`mint`] over an
//! injected [`Rng`] and an occupied set, the [`SplitMix64`] production
//! generator, and the loud [`MintError`]. The wordlist itself is **not**
//! an exported surface — the interface is the function.
//!
//! Like the linked binding, this seam promises pin-exact 0.x consumption
//! only (no semver stability). It is enumerated, alongside the command
//! surface, by `tests/command_surface_parity/`.

pub use crate::workspace::agent_name::mint::{MintError, Rng, SplitMix64, mint};
