//! **The seat's mark**: a ring, a disc, and the one wire between them.
//!
//! A window that names no icon wears whatever the desktop invents for it, and
//! what the desktop invents is the generic one every unnamed client gets. So
//! the seat carries a mark of its own — and it is *its own*: the engine's mark
//! belongs to the engine, and two programs wearing one picture is worse than
//! one program wearing none.
//!
//! **What the picture says.** A ring is the engine, held elsewhere and hollow
//! because nothing of it is here. A filled disc is the seat, which is here. Between them runs the one wire, in the dimmer of the two
//! inks, reaching into the ring rather than stopping at it. That is the whole
//! of what this crate is (`crate::channel` — *one wire to one engine*), and it
//! is three shapes because at 16 px a taskbar icon has room for three.
//!
//! # Two emissions, one geometry
//!
//! [`svg`] emits the checked-in vector source and [`rgba`] rasterizes the same
//! shape list for the toolkit's own icon call. They walk one list in one order,
//! so they are the same picture rather than two approximations of it — and
//! `tests::artifacts` pins the checked-in file against the generator byte for
//! byte, so `assets/lernie.svg` is a **derivation**, never a hand-edit.
//!
//! # Everything is an integer, and that is the design
//!
//! Every coordinate is a whole thousandth of the canvas, every inside-test is
//! an exact integer comparison, and the raster's coverage is a count of
//! subsamples rather than a distance passed through a curve. Three things fall
//! out of that and each is worth the arithmetic:
//!
//! - **The pinned artifact is reproducible.** A byte-for-byte test over a file
//!   whose numbers came out of floating point is a test that can disagree with
//!   itself on another target. Integers cannot.
//! - **Nothing narrows.** A float rasterizer ends in an `f32 as u8`, which is
//!   a cast the house lint set denies with no home for a suppression but the
//!   manifest — a crate-wide relaxation bought for one file.
//! - **No renderer is linked.** The mark is arithmetic, so this crate carries
//!   no image or vector dependency for a picture it can compute.
//!
//! # Where the mark actually shows, which is not where you would guess
//!
//! **Wayland has no protocol for a client to set its own window icon.** The
//! toolkit's icon call reaches X11 and nothing else; a Wayland compositor
//! resolves a window's mark by matching its **application id** against an
//! installed desktop entry, and then the entry's `Icon=` by NAME through the
//! hicolor theme. So the seat needs three things to agree — the app id
//! `crate::mark::APP_ID` that `main.rs` hands the toolkit, the entry's
//! `StartupWMClass`, and the basename of the installed SVG — and `make
//! icon-seats` is what lays the last two down. `tests::artifacts` pins the
//! agreement, because a mark that resolves nowhere fails silently and looks
//! exactly like not having one.
//!
//! **And it needs no PNG, which is why this repository still tracks no
//! binary.** Upstream laid sized PNGs beside the scalable source for shells
//! that will not read an SVG, and paid for them with the only entry its
//! disclosure gate's `BINARY_ALLOWED` has ever held. hicolor's scalable
//! directory is read by every desktop this seat is installed on; a sized raster
//! is a derivation nobody here consumes, and an allowlist entry for a file
//! nothing needs is a hole in the gate bought for nothing. The gate's allowlist
//! stays the empty set.

use shape::{SPAN, Shape};

/// The pixel loop.
mod raster;
/// The three primitives, and the two questions each answers.
mod shape;

/// **The application id**, which is the whole of what a Wayland compositor has
/// to go on. It is the crate's own name because the desktop entry and the
/// installed icon are named for the crate too, and the three agreeing is the
/// only reason any of them is seen.
pub const APP_ID: &str = "lernie";

/// One flat colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Ink {
    red: u8,
    green: u8,
    blue: u8,
}

impl Ink {
    /// The six-digit spelling the vector source wants.
    fn hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
    }
}

/// The two inks, and there are two on purpose: a third saturated colour is what
/// makes a mark read as decoration, and at 16 px it reads as noise.
const PHOSPHOR: Ink = Ink {
    red: 0x5f,
    green: 0xd1,
    blue: 0xb4,
};
const CASING: Ink = Ink {
    red: 0x2c,
    green: 0x61,
    blue: 0x57,
};

/// Where the two nodes sit, in thousandths. The wire is written from these
/// rather than repeating them, so a node cannot be moved without its wire.
const ENGINE: (i64, i64) = (660, 340);
const SEAT: (i64, i64) = (240, 790);

/// **The mark, back to front.** The wire is laid first so both nodes sit over
/// it — which is what lets the wire run to the engine's centre and read as
/// reaching into it rather than stopping short.
fn shapes() -> Vec<Shape> {
    vec![
        Shape::Bar {
            ax: SEAT.0,
            ay: SEAT.1,
            bx: ENGINE.0,
            by: ENGINE.1,
            half: 42,
            ink: CASING,
        },
        Shape::Ring {
            cx: ENGINE.0,
            cy: ENGINE.1,
            r: 200,
            half: 55,
            ink: PHOSPHOR,
        },
        Shape::Disc {
            cx: SEAT.0,
            cy: SEAT.1,
            r: 140,
            ink: PHOSPHOR,
        },
    ]
}

/// **The vector source**, which is the file `assets/lernie.svg` must equal.
///
/// One element per shape in the same order, on a canvas whose `viewBox` is the
/// unit every coordinate above is written in — so the file reads as the shape
/// list does, and a change to one is visible in the other.
pub fn svg() -> String {
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {SPAN} {SPAN}\" \
         width=\"{SPAN}\" height=\"{SPAN}\">\n"
    );
    for shape in shapes() {
        out.push_str(&shape.element());
        out.push('\n');
    }
    out.push_str("</svg>\n");
    out
}

/// **The mark rasterized `size`×`size`** as straight-alpha RGBA8, row-major
/// from the top-left — the format the toolkit's icon call takes.
pub fn rgba(size: u16) -> Vec<u8> {
    raster::rgba(&shapes(), size)
}

#[cfg(test)]
mod tests;
