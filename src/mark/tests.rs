//! The mark: the artifacts it pins, the three spellings that have to agree,
//! the hues' one home, and the picture the raster actually paints. The walk's
//! own promises are `walk`'s.

use super::{APP_ID, HYDRA, MAIN_R, NODE_R, VOID_DEEP, deep, icon, raster, rgba, svg};
use crate::ui::theme;

/// The walk, verified where the phone verifies it.
mod walk;

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
/// Byte for byte, which the emitter's fixed precision is what makes safe: a
/// generator that printed floats at full length could disagree with its own
/// output on another target, and this test would then fail for a reason that
/// had nothing to do with the picture.
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
/// The COUNT is what the substitution rides on: it replaces every line
/// matching `^Exec=`, so a second one would be rewritten to the same path and
/// the entry would carry the key twice — and a `.desktop` with a duplicate key
/// in one group is malformed, which launchers answer by ignoring the file.
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

/// The vector source is one element per shape, in the icon's own order — the
/// round ground first, so the file reads as the list does and a reordering
/// shows up in both. And every number in it is at the one fixed precision.
#[test]
fn every_shape_is_one_element_and_the_order_is_the_icon_s() {
    let source = svg();
    let elements: Vec<&str> = source
        .lines()
        .filter(|line| line.starts_with("  <"))
        .collect();
    assert_eq!(elements.len(), icon().len());
    assert_eq!(
        elements[0], "  <circle cx=\"500.0\" cy=\"500.0\" r=\"500.0\" fill=\"#0a080f\"/>",
        "the ground is the void, round, and under everything"
    );
    assert!(
        elements[1].contains("stroke-linejoin=\"round\""),
        "{source}"
    );
    assert!(
        elements.last().unwrap().contains(" Z\" fill=\"#0a080f\""),
        "the pupil is last"
    );
    assert!(source.ends_with("</svg>\n"), "{source}");
    let attributes = elements
        .iter()
        .flat_map(|e| e.split('"').skip(1).step_by(2));
    for value in attributes.filter(|v| v.contains('.')) {
        for figure in value
            .split([' ', ',', 'M', 'L'])
            .filter(|f| !f.is_empty() && f != &"Z")
        {
            assert_eq!(
                figure.split('.').nth(1).map(str::len),
                Some(1),
                "{figure} in {value}"
            );
        }
    }
}

/// **The two hues have one home each, and this module restates them.** The
/// walk is pure and links no toolkit, so it cannot read `ui::theme`; this is
/// what holds the two spellings together.
#[test]
fn the_palette_hue_is_the_theme_s_attention_and_the_void_is_its_ground() {
    assert_eq!(
        theme::accent(theme::State::Attention).to_array()[..3],
        HYDRA
    );
    assert_eq!(theme::GROUND.to_array()[..3], VOID_DEEP);
}

/// **The integer drive lands on the phone's bytes.** The phone computes
/// `deep` in `f32` and truncates to a byte; these are the values it emits for
/// the conductor and the casing, and a port that drifted from them would be
/// two marks in two hues.
#[test]
fn deepening_in_hundredths_lands_on_the_phone_s_own_bytes() {
    assert_eq!(deep(HYDRA, super::PHOSPHOR), [32, 255, 108]);
    assert_eq!(deep(HYDRA, super::CASING), [12, 99, 42]);
}

/// The pixel at unit coordinates (`x`, `y`) of an `n`×`n` raster.
fn spot(px: &[u8], n: u16, x: f32, y: f32) -> [u8; 4] {
    let at = |v: f32| {
        usize::from(
            (0..n)
                .find(|i| f32::from(*i + 1) / f32::from(n) > v)
                .unwrap(),
        )
    };
    let i = (at(y) * usize::from(n) + at(x)) * 4;
    px[i..i + 4].try_into().unwrap()
}

fn opaque(hue: [u8; 3]) -> [u8; 4] {
    [hue[0], hue[1], hue[2], 255]
}

/// **The raster is the shape list and nothing else**, asserted where the facts
/// of the picture are: outside the round ground is the canvas, inside it is
/// the void, the centre is the pupil's void over the eye, a node is the
/// phosphor, and the casing shows at a trace's edge.
#[test]
fn the_raster_paints_the_void_ground_the_pupil_the_nodes_and_the_casing() {
    let n = 128_u16;
    let px = rgba(n);
    assert_eq!(px.len(), usize::from(n) * usize::from(n) * 4);
    let phosphor = opaque(deep(HYDRA, super::PHOSPHOR));
    assert_eq!(
        spot(&px, n, 0.02, 0.02),
        [0; 4],
        "a corner is off the ground"
    );
    assert_eq!(
        spot(&px, n, 0.2, 0.2),
        opaque(VOID_DEEP),
        "the ground is void"
    );
    assert_eq!(
        spot(&px, n, 0.5, 0.5),
        opaque(VOID_DEEP),
        "the pupil is void"
    );
    assert_eq!(spot(&px, n, 0.5 + MAIN_R * 0.8, 0.5), phosphor, "the eye");
    assert_eq!(spot(&px, n, 0.5, 0.5 + MAIN_R + NODE_R), phosphor, "a node");
    let casing = opaque(deep(HYDRA, super::CASING));
    assert!(px.chunks_exact(4).any(|pixel| pixel == casing), "no casing");
}

/// **A size with nothing in it is nothing**, which is the loop not taken
/// rather than a case.
#[test]
fn a_raster_of_no_size_is_empty() {
    assert!(rgba(0).is_empty());
}

/// Every edge the mark has, at a small size: the antialiased band exists at
/// all (a picture with only opaque and transparent pixels is one the subsample
/// grid never split), and every pixel is a mean of hues the walk owns, so no
/// channel can exceed the brightest of them.
#[test]
fn the_edges_are_antialiased_and_every_pixel_is_a_mean_of_the_walk_s_hues() {
    let px = rgba(32);
    let soft = px
        .chunks_exact(4)
        .filter(|p| p[3] > 0 && p[3] < 255)
        .count();
    assert!(soft > 0, "nothing on the mark's edges was ever split");
    assert!(px.chunks_exact(4).all(|p| p[0] <= 32 && p[2] <= 108));
}

/// The sample count is the grid squared — two constants, because a cast-free
/// `const` cannot derive one from the other, held together here.
#[test]
fn the_sample_count_is_the_grid_squared() {
    let (sub, samples) = raster::samples();
    assert_eq!(u32::from(sub).pow(2), samples);
}
