//! **The fallback face** (bl-3956, DESIGN §4.43): CJK the bundled faces lack,
//! read off the operator's own system at window start.
//!
//! egui's four bundled faces hold Latin, a few symbols and 887 emoji code
//! points and no ideograph, kana or hangul, so CJK paints as tofu. The seat
//! ships no font of its own (a tracked binary would end the leak gate's empty
//! allowlist); it reads the first file of [`FALLBACKS`] that exists and pushes
//! it LAST in both families, so a glyph the bundled faces have keeps its face
//! and only what was tofu gains one. A box with none of them keeps today's
//! face and says nothing.

use egui::{FontData, FontDefinitions, FontFamily};

/// **Where a CJK face is kept**, tried in order, each with the face to take
/// inside a collection. The single-face Droid file is first because it is the
/// smaller (~4 MB against ~19.5 MB) and so the faster to read; the Noto
/// collection's face 0 is its Japanese one. Colour fonts (Noto Color Emoji,
/// Apple Color Emoji) are not listed: epaint draws no colour glyph.
pub const FALLBACKS: &[(&str, u32)] = &[
    (
        "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
        0,
    ),
    (
        "/usr/share/fonts/google-droid-sans-fonts/DroidSansFallbackFull.ttf",
        0,
    ),
    ("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", 0),
    ("/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc", 0),
    (
        "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
        0,
    ),
    ("/System/Library/Fonts/PingFang.ttc", 0),
    ("/System/Library/Fonts/Hiragino Sans GB.ttc", 0),
];

/// The name the fallback is filed under in [`FontDefinitions::font_data`].
pub const FALLBACK: &str = "fallback";

/// **egui's faces, plus the first readable font of `faces` last in both
/// families.** The window passes [`FALLBACKS`]; the paint harness installs
/// nothing, so what a test paints never depends on the box's fonts.
pub fn fonts(faces: &[(&str, u32)]) -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    if let Some(data) = faces.iter().find_map(|&(path, index)| read(path, index)) {
        fonts.font_data.insert(FALLBACK.to_owned(), data.into());
        for family in [FontFamily::Proportional, FontFamily::Monospace] {
            fonts
                .families
                .entry(family)
                .or_default()
                .push(FALLBACK.to_owned());
        }
    }
    fonts
}

/// One file as a face, or nothing: missing, unreadable, or not a font. The
/// check is the sfnt tag — TrueType, OpenType or a collection — because egui
/// panics on bytes it cannot parse, and a file at a font's path that is not
/// one must cost a face, never the window.
fn read(path: &str, index: u32) -> Option<FontData> {
    let bytes = std::fs::read(path).ok()?;
    let tag = bytes.get(..4)?;
    [&[0, 1, 0, 0][..], b"true", b"OTTO", b"ttcf"]
        .contains(&tag)
        .then(|| FontData {
            index,
            ..FontData::from_owned(bytes)
        })
}

#[cfg(test)]
mod tests {
    use super::{FALLBACK, fonts};
    use crate::test_support::Scratch;
    use egui::FontFamily;

    fn bundled() -> egui::FontDefinitions {
        egui::FontDefinitions::default()
    }

    #[test]
    fn no_file_on_the_list_keeps_the_bundled_faces() {
        let scratch = Scratch::new();
        let gone = scratch.join("gone.ttf");
        assert_eq!(fonts(&[(gone.to_str().unwrap(), 0)]), bundled());
        assert_eq!(fonts(&[]), bundled());
    }

    #[test]
    fn a_file_that_is_not_a_font_is_passed_over() {
        let scratch = Scratch::new();
        let (text, empty) = (scratch.join("words.ttf"), scratch.join("empty.ttf"));
        std::fs::write(&text, "not a font at all").unwrap();
        std::fs::write(&empty, "").unwrap();
        let list = [(text.to_str().unwrap(), 0), (empty.to_str().unwrap(), 0)];
        assert_eq!(fonts(&list), bundled());
    }

    /// The bundled monospace face written to a file stands in for a system
    /// one: the first READABLE font wins, filed last in both families, and
    /// egui builds a context with it.
    #[test]
    fn the_first_font_on_the_list_is_the_last_face_of_both_families() {
        let scratch = Scratch::new();
        let (junk, font) = (scratch.join("junk.ttf"), scratch.join("face.ttf"));
        std::fs::write(&junk, "nope").unwrap();
        std::fs::write(&font, &*bundled().font_data["Hack"].font).unwrap();
        let path = font.to_str().unwrap();
        let got = fonts(&[(junk.to_str().unwrap(), 0), (path, 0), (path, 7)]);
        assert_eq!(got.font_data[FALLBACK].index, 0);
        for family in [FontFamily::Proportional, FontFamily::Monospace] {
            let mut want = bundled().families[&family].clone();
            want.push(FALLBACK.to_owned());
            assert_eq!(got.families[&family], want);
        }
        let ctx = egui::Context::default();
        ctx.set_fonts(got);
        let _ = ctx.run(egui::RawInput::default(), |_| {});
    }
}
