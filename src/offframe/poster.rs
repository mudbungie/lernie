//! **The poster**: one pass over what a frame composed.
//!
//! It is a thread of its own and not a leg of the asker's pass, because an act
//! must not wait behind a read that is mid-flight. An operator who has pressed
//! send has already decided; making them wait for a roster refresh to finish
//! first is the seat inserting itself into their intent.
//!
//! **A receipt is a frame like any other**, so it goes back through the same
//! door every answer does and lands as content or as the notice bar. Nothing
//! here reads what an act earned.

use std::path::Path;

use crate::state::{Link, Said};

/// Send everything the frame composed since the last pass.
///
/// The receipt is stamped with the **aimed** channel, which is where a composed
/// gesture came from. A gesture composed with no aim is still sent — the
/// address it carries is what routes it, and the address is the whole of what
/// routing needs — and its receipt is stamped with a channel that names
/// nothing, because a stamp this seat cannot honestly make is not one it should
/// invent.
pub fn tick(link: &Link, root: &Path) {
    let standing = link.standing();
    let channel = standing
        .aimed()
        .map(|(channel, _)| channel)
        .unwrap_or_default();
    for envelope in link.compose() {
        match crate::seat::route(root, &envelope).and_then(|(open, carried)| open.ask(&carried)) {
            Ok(stream) => {
                for frame in stream {
                    link.heard(&channel, Said::Frame(frame));
                }
            }
            Err(why) => link.heard(&channel, Said::Unreachable(why)),
        }
    }
}

#[cfg(test)]
mod tests;
