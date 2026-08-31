//! The seat's suite, and the scaffolding its two halves share.
//!
//! Split at the line cap along the seam the module itself has: [`routing`] is
//! which engine a gesture reaches and what it carries there, [`refusals`] is
//! what comes back when it cannot reach one, [`listing`] is what this box says
//! it holds without dialling any of it.

use super::{ask, listing, route};
use crate::test_support::wire::{entry, flat, wired, yes};

/// What the box says it holds, without dialling any of it.
mod listing;
/// What a box that cannot reach an engine says, and what an answer is worth.
mod refusals;
/// Which engine a gesture reaches and what it carries there.
mod routing;
