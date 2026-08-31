//! The mark: the artifacts it pins, the three spellings that have to agree, and
//! the picture the raster actually paints.

use super::{APP_ID, CASING, ENGINE, PHOSPHOR, SEAT, rgba, shapes, svg};

/// One tracked asset, read out of the source tree rather than out of a target
/// directory: these files are the repository's, and the test is about what is
/// checked in.
fn asset(name: &str) -> String {
    let at = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join(name);
    std::fs::read_to_string(&at).unwrap_or_else(|why| panic!("{}: {why}", at.display()))
}

/// **The checked-in vector source is a derivation, not a hand-edit.**
///
/// Byte for byte, which the mark's integer geometry is what makes safe: a
/// generator that emitted formatted floats could disagree with its own output
/// on another target, and this test would then fail for a reason that had
/// nothing to do with the picture.
#[test]
fn the_tracked_vector_source_is_exactly_what_the_generator_emits() {
    assert_eq!(
        asset(&format!("{APP_ID}.svg")),
        svg(),
        "assets/{APP_ID}.svg is stale — re-emit it with `make icon`, never by hand"
    );
}

/// **Three spellings, and nothing at runtime can check them.** A Wayland
/// compositor matches the window's app id against the desktop entry and reads
/// that entry's `Icon=` by name through the theme, so a disagreement between
/// any two of them is a mark that resolves nowhere — which looks exactly like
/// not having one.
#[test]
fn the_app_id_the_desktop_entry_and_the_icon_name_are_one_word() {
    let entry = asset(&format!("{APP_ID}.desktop"));
    for line in [
        format!("Icon={APP_ID}"),
        format!("StartupWMClass={APP_ID}"),
        format!("Exec={APP_ID}"),
    ] {
        assert!(
            entry.lines().any(|held| held == line),
            "the desktop entry has no {line:?}:\n{entry}"
        );
    }
}

/// **The tracked entry's `Exec` is the bare name, exactly once.**
///
/// `make icon-seats` rewrites `Exec=` into the copy it INSTALLS, because a
/// desktop environment resolves that line out of the session's own environment
/// and a bare name is silently unlaunchable wherever the binary's directory
/// reached `PATH` through a shell profile. The tracked file stays generic — it
/// is one repository's source for every box, and a real absolute path in it
/// would be a disclosure as well as a lie everywhere else.
///
/// Two halves, and the second is the one no reader would think to keep. The
/// bare spelling above is what the test before this one already pins. The
/// COUNT is what the substitution rides on: it replaces every line matching
/// `^Exec=`, so a second one would be rewritten to the same path and the entry
/// would carry the key twice — and a `.desktop` with a duplicate key in one
/// group is malformed, which launchers answer by ignoring the file rather than
/// by complaining about it.
#[test]
fn the_tracked_entry_carries_exactly_one_exec_and_it_is_generic() {
    let entry = asset(&format!("{APP_ID}.desktop"));
    let execs: Vec<&str> = entry
        .lines()
        .filter(|line| line.starts_with("Exec="))
        .collect();
    assert_eq!(
        execs,
        vec![format!("Exec={APP_ID}")],
        "the installed copy is where an absolute Exec belongs, and there is one \
         of them — `make icon-seats` rewrites every ^Exec= line:\n{entry}"
    );
}

/// The vector source is one element per shape, in the shape list's own order —
/// so the file reads as the list does and a reordering shows up in both.
#[test]
fn every_primitive_is_one_element_and_the_order_is_the_list_s() {
    let source = svg();
    let elements: Vec<&str> = source
        .lines()
        .filter(|line| line.starts_with("  <"))
        .collect();
    assert_eq!(elements.len(), shapes().len());
    assert!(elements[0].starts_with("  <line "), "{source}");
    assert!(elements[1].contains("fill=\"none\""), "{source}");
    assert!(elements[2].contains("fill=\"#5fd1b4\""), "{source}");
    assert!(source.ends_with("</svg>\n"), "{source}");
}

/// The pixel at unit coordinates (`x`, `y`) of an `n`×`n` raster.
fn spot(px: &[u8], n: u16, x: i64, y: i64) -> [u8; 4] {
    let at = |v: i64| usize::try_from(v * i64::from(n) / 1000).unwrap_or(0);
    let i = (at(y) * usize::from(n) + at(x)) * 4;
    let taken = px.get(i..i + 4).unwrap_or(&[0; 4]);
    [taken[0], taken[1], taken[2], taken[3]]
}

/// **The raster is the shape list and nothing else**, asserted where the four
/// facts of the picture are: the canvas is transparent, the seat is solid, the
/// engine is a RING and therefore hollow in the middle, and the wire between
/// them is the dimmer ink.
#[test]
fn the_raster_paints_a_hollow_engine_a_solid_seat_and_a_wire_between() {
    let n = 64_u16;
    let px = rgba(n);
    assert_eq!(px.len(), usize::from(n) * usize::from(n) * 4);

    assert_eq!(spot(&px, n, 20, 20), [0; 4], "the canvas is transparent");
    assert_eq!(
        spot(&px, n, SEAT.0, SEAT.1),
        [PHOSPHOR.red, PHOSPHOR.green, PHOSPHOR.blue, 255],
        "the seat is solid"
    );
    assert_eq!(
        spot(&px, n, ENGINE.0 + 73, ENGINE.1 + 68)[3],
        0,
        "the engine is a ring, so inside it is the canvas"
    );
    assert_eq!(
        spot(&px, n, ENGINE.0, ENGINE.1),
        [CASING.red, CASING.green, CASING.blue, 255],
        "and the wire runs to its centre rather than stopping at its edge"
    );
    assert_eq!(
        spot(&px, n, ENGINE.0, ENGINE.1 - 200),
        [PHOSPHOR.red, PHOSPHOR.green, PHOSPHOR.blue, 255],
        "and its stroke is on it"
    );
    assert_eq!(
        spot(
            &px,
            n,
            i64::midpoint(SEAT.0, ENGINE.0),
            i64::midpoint(SEAT.1, ENGINE.1)
        ),
        [CASING.red, CASING.green, CASING.blue, 255],
        "the wire is the dimmer ink"
    );
}

/// **A size with nothing in it is nothing**, which is the loop not taken rather
/// than a case: an icon of no pixels is not an error to refuse, it is a picture
/// with no room to be one.
#[test]
fn a_raster_of_no_size_is_empty() {
    assert!(rgba(0).is_empty());
}

/// Every edge the mark has, at the one size a taskbar actually uses. Two facts
/// ride on it: the antialiased band exists at all (a picture with only opaque
/// and transparent pixels is one the subsample grid never split), and no pixel
/// is ever partly one ink and partly the other's *neighbour* — every painted
/// pixel is a mean of the two inks and so lies between them.
#[test]
fn the_edges_are_antialiased_and_every_pixel_is_a_mean_of_the_two_inks() {
    let n = 16_u16;
    let px = rgba(n);
    let mut soft = 0;
    for pixel in px.chunks_exact(4) {
        let (red, alpha) = (pixel.first(), pixel.get(3));
        if alpha.copied().is_some_and(|a| a > 0 && a < 255) {
            soft += 1;
        }
        if alpha.copied().is_some_and(|a| a > 0) {
            let red = red.copied().unwrap_or(0);
            assert!(
                (CASING.red..=PHOSPHOR.red).contains(&red),
                "a pixel is neither ink nor a mean of them: {pixel:?}"
            );
        }
    }
    assert!(soft > 0, "nothing on the mark's edges was ever split");
}
