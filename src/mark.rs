//! **The seat's mark**: the congeries as a circuit — the same picture the
//! phone seat wears, because the two seats are two faces of one product
//! (operator ruling, bl-708e). Until then this module argued the opposite —
//! that a program must not share a picture with another — and drew a ring, a
//! disc and a wire of its own. That doctrine is withdrawn: the SERVER keeps a
//! mark of its own; every seat of it wears this one.
//!
//! **What the picture is.** Three circles sit tangent to a central one, 120°
//! apart with one at bottom dead centre. From each, an arm runs 60° of arc to
//! a further circle; the circles between ride that arc, joined by a trace of
//! dim casing under a bright phosphor conductor. The centre carries a slit
//! pupil in the void's own black. Two hues, both driven from one palette
//! entry — the theme's attention green — on the theme's ground.
//!
//! **Everything is compass work.** No spline, no easing: an arc is named by
//! two endpoints and the height of its apex above the chord (`arc::arc`). The
//! pupil is a *lune*, the sliver between two arcs sharing a pair of endpoints,
//! pointed at each end because that is where the arcs meet — and it is
//! **filled, not punched**: cut as negative space it showed the traces
//! converging beneath it.
//!
//! # One walk, three emissions
//!
//! [`mark`] is the shape list, and it is the phone's walk constant for
//! constant: the same `Trace` / `Lune` / `Disc` vocabulary in the same layer
//! order, so the two seats' marks are one geometry rather than two drawings
//! of one idea. [`icon`] is that list over a round ground, which is the
//! phone's launcher mask — so the two icons read alike on a desk and on a home
//! screen. [`svg`] emits the checked-in vector source from `icon`, [`rgba`]
//! rasterizes the same list for the toolkit's own icon call, and the window
//! says the bare `mark` in egui's words (`ui::shell`). `tests::artifacts`
//! pins the checked-in file against the generator byte for byte, so
//! `assets/lernie.svg` is a **derivation**, never a hand-edit.
//!
//! # The walk is `f32`, and nothing narrows
//!
//! The phone's walk is single-precision trigonometry and so is this one —
//! porting it to integers would be a second geometry wearing the first's name.
//! What the house lint set denies is the `as` cast, and there is no home for a
//! suppression but the manifest, so the port avoids casts by construction:
//! pixel coordinates widen through `f32::from(u16)`, counts narrow through
//! `u8::try_from`, the raster's coverage is an integer count of subsamples
//! rather than a distance passed through a curve, and the hue drive
//! ([`deep`]) is fixed-point integer arithmetic that lands on the phone's own
//! bytes. The one place a float becomes text is the vector source, written at
//! a fixed precision so a last-bit difference between targets does not reach
//! the file — the same discipline the phone's drawable keeps.
//!
//! # Where the mark actually shows, which is not where you would guess
//!
//! **Wayland has no protocol for a client to set its own window icon.** The
//! toolkit's icon call reaches X11 and nothing else; a Wayland compositor
//! resolves a window's mark by matching its **application id** against an
//! installed desktop entry, and then the entry's `Icon=` by NAME through the
//! hicolor theme. So the seat needs three things to agree — the app id
//! [`APP_ID`] that `main.rs` hands the toolkit, the entry's `StartupWMClass`,
//! and the basename of the installed SVG — and `make icon-seats` is what lays
//! the last two down. `tests::artifacts` pins the agreement, because a mark
//! that resolves nowhere fails silently and looks exactly like not having one.
//!
//! **And it needs no PNG, which is why this repository still tracks no
//! binary.** hicolor's scalable directory is read by every desktop this seat
//! is installed on; the SVG is text the disclosure gate reads, so the gate's
//! binary allowlist stays the empty set.

/// Compass work: arcs, lunes, and the shape vocabulary.
mod arc;
/// The pixel loop.
mod raster;
/// The vector emitter.
mod svg;

use arc::lune;
pub use arc::{Point, Rib, Shape};

#[cfg(test)]
mod tests;

/// **The application id**, which is the whole of what a Wayland compositor has
/// to go on. It is the crate's own name because the desktop entry and the
/// installed icon are named for the crate too, and the three agreeing is the
/// only reason any of them is seen.
pub const APP_ID: &str = "lernie";

/// A flat fill, RGB — the walk's whole colour vocabulary. Every fill is
/// opaque; transparency is the canvas's, and coverage is the raster's.
pub type Hue = [u8; 3];

/// The one palette entry the mark is built from, and the void its pupil and
/// ground are filled with — the theme's attention green and its ground,
/// restated here because the walk is pure and links no toolkit. `tests`
/// holds the two spellings to one another.
const HYDRA: Hue = [110, 222, 148];
const VOID_DEEP: Hue = [13, 16, 21];

/// Arms, and the degrees between them — the triskele's three-fold turn.
const ARMS: u8 = 3;
const TURN: f32 = 120.0;
/// Points sampled along one arc. Divisible by `CIRCLES - 1`, so the seats
/// fall on sampled points exactly rather than between them.
const STEPS: u16 = 48;
/// Where the first arm's tangent circle sits, in degrees: bottom dead centre.
const BASE: f32 = 90.0;
/// The middle circle, and the radius of every other circle — all equal.
const MAIN_R: f32 = 0.132;
const NODE_R: f32 = 0.048;
/// An arm ends this far out, and this many degrees of arc from where it
/// began — the pair that puts the top-right arm's last circle straight up.
const END_R: f32 = 0.398;
const SWEEP: f32 = 60.0;
/// Circles an arm, counting the tangent one it starts on and the one it ends
/// on.
const CIRCLES: u8 = 3;
/// **The only free parameter.** The sagitta of an arm's arc, as a fraction
/// of the chord between its two pinned ends: at 0.448 the arm leaves its
/// tangent circle at exactly 45° off the radial.
const SWELL: f32 = 0.448;
/// The dim casing, and the bright conductor laid down its middle.
const CASING_W: f32 = 0.054;
const CONDUCTOR_W: f32 = 0.016;
/// The pupil's half-height and bulge, as fractions of the middle circle.
const PUPIL_LONG: f32 = 0.78;
const PUPIL_BULGE: f32 = 0.30;
/// How hard [`deep`] drives the one hue, in hundredths: the conductor past
/// the hue's own peak until it is phosphor, the casing well under it.
const PHOSPHOR: i64 = 115;
const CASING: i64 = 45;
/// How much of the palette hue's white component [`deep`] strips, in
/// hundredths.
const WHITE_CUT: i64 = 85;

/// The palette hue with its white component stripped and its value scaled by
/// `value` hundredths. The badge hue is pastel — tuned to read as a small
/// mark on a dark panel — and a logo wants it saturated; driven past 1.0 the
/// green goes phosphor.
///
/// The phone computes this in `f32` and truncates; here the same sum is taken
/// in hundredths over `i64`, truncating the same way, and lands on the same
/// bytes — `tests` pins them — without the `as u8` the truncation would
/// otherwise end in.
fn deep(hue: Hue, value: i64) -> Hue {
    let [red, green, blue] = hue.map(|channel| i64::from(channel) * 100);
    let high = red.max(green).max(blue);
    let low = red.min(green).min(blue) * WHITE_CUT / 100;
    let pure = |channel: i64| {
        let driven = high * (channel - low) * value / (high - low) / 10_000;
        u8::try_from(driven.clamp(0, 255)).unwrap_or(u8::MAX)
    };
    [pure(red), pure(green), pure(blue)]
}

/// The unit-square point `radius` out from the centre at `degrees`.
fn polar(radius: f32, degrees: f32) -> Point {
    let turn = degrees.to_radians();
    (
        radius.mul_add(turn.cos(), 0.5),
        radius.mul_add(turn.sin(), 0.5),
    )
}

/// One arm's two **pinned** points: a circle tangent to the middle one, and
/// a circle [`SWEEP`] degrees of arc counter-clockwise of it at [`END_R`].
fn pins(turn: u8) -> (Point, Point) {
    let base = BASE + TURN * f32::from(turn);
    (polar(MAIN_R + NODE_R, base), polar(END_R, base - SWEEP))
}

/// The arc between them, bowed by [`SWELL`] of the chord.
fn trace(turn: u8) -> Vec<Point> {
    let (from, to) = pins(turn);
    let chord = (to.0 - from.0).hypot(to.1 - from.1);
    arc::arc(from, to, SWELL * chord)
}

/// Where an arm's circles sit: [`CIRCLES`] points spaced evenly along its
/// arc. Even spacing on a circular arc means equal chords, so the legs
/// between them come out equal without anyone asking.
fn seats(turn: u8) -> Vec<Point> {
    let path = trace(turn);
    let stride = usize::from(STEPS / u16::from(CIRCLES - 1));
    (0..CIRCLES)
        .filter_map(|step| path.get(usize::from(step) * stride).copied())
        .collect()
}

/// A flat circle in the walk's words.
fn disc((cx, cy): Point, radius: f32, fill: Hue) -> Shape {
    Shape::Disc {
        cx,
        cy,
        radius,
        fill,
    }
}

/// **The whole mark at rest, back to front — by layer, not by arm**: every
/// casing goes down, then every conductor, then every circle, then the eye
/// over the place the arms converge. Drawing each arm complete in turn looks
/// equivalent and is not — the arms overlap near the middle, so a later
/// one's casing would paint over an earlier one's conductor.
///
/// Coordinates are the unit square; a renderer maps them to whatever rect it
/// holds, keeping the square (the mark is round, and a stretched one is a
/// different picture). The canvas is whatever is under it: the window's
/// panel, or [`icon`]'s ground.
pub fn mark() -> Vec<Shape> {
    let (conductor, casing) = (deep(HYDRA, PHOSPHOR), deep(HYDRA, CASING));
    let traces: Vec<Vec<Point>> = (0..ARMS).map(trace).collect();
    let band = |width, fill| {
        traces.iter().map(move |path| Shape::Trace {
            path: path.clone(),
            width,
            fill,
        })
    };
    let mut out: Vec<Shape> = band(CASING_W, casing)
        .chain(band(CONDUCTOR_W, conductor))
        .collect();
    out.extend(
        (0..ARMS)
            .flat_map(seats)
            .map(|seat| disc(seat, NODE_R, conductor)),
    );
    out.push(disc((0.5, 0.5), MAIN_R, conductor));
    let reach = MAIN_R * PUPIL_LONG;
    out.push(lune(
        (0.5, 0.5 - reach),
        (0.5, 0.5 + reach),
        MAIN_R * PUPIL_BULGE,
        -MAIN_R * PUPIL_BULGE,
        VOID_DEEP,
    ));
    out
}

/// **The icon**: the mark over a round ground in the void, which is the shape
/// a phone's launcher mask cuts — so a desk and a home screen show one
/// picture. The ground is the first shape because the walk is painted back to
/// front, and the mark's furthest ink (`END_R + NODE_R` = 0.446 from the
/// centre) sits inside the ground's 0.5.
pub fn icon() -> Vec<Shape> {
    let mut out = vec![disc((0.5, 0.5), 0.5, VOID_DEEP)];
    out.extend(mark());
    out
}

/// **The vector source**, which is the file `assets/lernie.svg` must equal:
/// one element per shape of [`icon`], in the walk's order.
pub fn svg() -> String {
    svg::document(&icon())
}

/// **The icon rasterized `size`×`size`** as straight-alpha RGBA8, row-major
/// from the top-left — the format the toolkit's icon call takes.
pub fn rgba(size: u16) -> Vec<u8> {
    raster::rgba(&icon(), size)
}
