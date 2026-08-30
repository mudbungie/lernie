//! The seat's suite, and the scaffolding its two halves share.
//!
//! Split at the line cap along the seam the module itself has: [`routing`] is
//! which engine a gesture reaches and what it carries there, [`listing`] is
//! what this box says it holds without dialling any of it.

use super::{ask, listing, route};
use crate::test_support::wire::{entry, flat, wired, yes};

/// What the box says it holds, without dialling any of it.
mod listing;
/// Which engine a gesture reaches and what it carries there.
mod routing;
