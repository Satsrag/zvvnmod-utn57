//! The detached-suffix boundary, in both directions.
//!
//! ZVVNMOD writes a detached suffix as `U+202F`, and an ordinary word space as
//! `U+0020`. UTN #57 writes the same boundary as `MVS`. Keeping the two spaces
//! apart is what lets the boundary survive a conversion: a hub that spelled both
//! with `U+0020` could not be read back without guessing which spaces separate
//! words and which separate a stem from its suffix.
//!
//! The baselines are meco's own hub text — `meco translate --from delehi --to
//! zvvnmod`, meco 0.4.0 — with its separator respelled `U+202F`. Every other
//! code below is meco's, character for character. A round trip through this
//! crate would be no baseline at all: before this contract existed, `U+180E`
//! passed through unchanged in both directions, so `utn57 → zvvnmod → utn57` was
//! stable while both halves leaked a UTN #57 control into ZVVNMOD text.

use zvvnmod_utn57::{convert_utn57_to_zvvnmod, convert_zvvnmod_to_utn57};

/// Spell a string as its code points.
fn hex(text: &str) -> String {
    text.chars()
        .map(|character| format!("{:04X}", character as u32))
        .collect::<Vec<_>>()
        .join(" ")
}

/// ᠲᠠᠯ᠎ᠠ ᠶᠢᠨ — `tal-a-yin`, a chachlag ᠠ and a consonant-initial detached suffix.
const TAL_A_YIN_HUB: &str = "E042 E005 E03B E00D 202F E04D E006 E00C";

/// ᠮᠣᠩᠭᠣᠯ ᠤᠨ — `mongol-un`, a vowel-initial detached suffix.
const MONGOL_UN_HUB: &str = "E036 E008 E005 E031 E028 E028 E008 E03B 202F E001 E00C";

#[test]
fn a_detached_suffix_boundary_is_a_narrow_no_break_space() {
    // ᠲᠠᠯ᠎ᠠ + NNBSP + ᠶᠢᠨ.
    assert_eq!(
        hex(&convert_utn57_to_zvvnmod(
            "\u{1832}\u{1820}\u{182F}\u{180E}\u{1820}\u{202F}\u{1836}\u{1822}\u{1828}"
        )
        .unwrap()),
        TAL_A_YIN_HUB
    );
    // ᠮᠣᠩᠭᠣᠯ + NNBSP + ᠤᠨ.
    assert_eq!(
        hex(&convert_utn57_to_zvvnmod(
            "\u{182E}\u{1823}\u{1829}\u{182D}\u{1823}\u{182F}\u{202F}\u{1824}\u{1828}"
        )
        .unwrap()),
        MONGOL_UN_HUB
    );
}

#[test]
fn an_mvs_spelling_of_the_same_boundary_reaches_the_same_hub_text() {
    // The same word with MVS in place of the NNBSP. UTN #57 has one reviewed
    // boundary control, so both spellings converge on one ZVVNMOD text.
    assert_eq!(
        hex(&convert_utn57_to_zvvnmod(
            "\u{1832}\u{1820}\u{182F}\u{180E}\u{1820}\u{180E}\u{1836}\u{1822}\u{1828}"
        )
        .unwrap()),
        TAL_A_YIN_HUB
    );
}

#[test]
fn a_lone_boundary_converts_in_both_directions() {
    assert_eq!(convert_utn57_to_zvvnmod("\u{180E}").unwrap(), "\u{202F}");
    assert_eq!(convert_utn57_to_zvvnmod("\u{202F}").unwrap(), "\u{202F}");
    assert_eq!(convert_zvvnmod_to_utn57("\u{202F}").unwrap(), "\u{180E}");
}

#[test]
fn a_boundary_before_a_consonant_initial_suffix_survives_on_its_own() {
    // ᠶᠢᠨ behind a boundary, with no stem in front of it.
    assert_eq!(
        hex(&convert_utn57_to_zvvnmod("\u{180E}\u{1836}\u{1822}\u{1828}").unwrap()),
        "202F E04D E006 E00C"
    );
}

#[test]
fn a_vowel_initial_detached_suffix_keeps_its_merged_chachlag_spelling() {
    // A boundary before a chachlag ᠠ/ᠡ is carried by the merged chachlag glyph
    // itself, so no separator is emitted. Both baselines are meco's hub text.
    // ᠲᠠᠯ᠎ᠠ
    assert_eq!(
        hex(&convert_utn57_to_zvvnmod("\u{1832}\u{1820}\u{182F}\u{180E}\u{1820}").unwrap()),
        "E042 E005 E03B E00D"
    );
    // ᠰᠢᠨ᠎ᠡ
    assert_eq!(
        hex(&convert_utn57_to_zvvnmod("\u{1830}\u{1822}\u{1828}\u{180E}\u{1821}").unwrap()),
        "E03C E006 E077"
    );
}

/// The property the two directions owe: a detached suffix boundary reaches the
/// far side of a conversion, for consonant-initial and vowel-initial suffixes
/// alike.
#[test]
fn a_detached_suffix_boundary_survives_a_conversion_in_both_directions() {
    let words = [
        // ᠲᠠᠯ᠎ᠠ ᠶᠢᠨ — consonant-initial, and a chachlag ᠠ ahead of it.
        "\u{1832}\u{1820}\u{182F}\u{180E}\u{1820}\u{202F}\u{1836}\u{1822}\u{1828}",
        // ᠮᠣᠩᠭᠣᠯ ᠤᠨ — vowel-initial.
        "\u{182E}\u{1823}\u{1829}\u{182D}\u{1823}\u{182F}\u{202F}\u{1824}\u{1828}",
        // ᠬᠡᠯᠡ ᠪᠡᠨ — a bowed consonant, which recomposes into a merged glyph.
        "\u{182C}\u{1821}\u{182F}\u{1821}\u{202F}\u{182A}\u{1821}\u{1828}",
    ];
    for word in words {
        let zvvnmod = convert_utn57_to_zvvnmod(word).unwrap();
        assert!(
            zvvnmod.contains('\u{202F}'),
            "{} lost its boundary: {}",
            hex(word),
            hex(&zvvnmod)
        );

        let utn57 = convert_zvvnmod_to_utn57(&zvvnmod).unwrap();
        assert!(
            utn57.contains('\u{180E}'),
            "{} lost its boundary on the way back: {}",
            hex(&zvvnmod),
            hex(&utn57)
        );
    }
}

#[test]
fn a_word_space_is_not_a_detached_suffix_boundary() {
    // ᠲᠠᠯ᠎ᠠ ᠶᠢᠨ written as two words. ZVVNMOD keeps the space, and ᠶ takes its
    // word-initial glyph E050 rather than the suffix-initial E04D — meco's own
    // hub text for the same two words.
    assert_eq!(
        hex(&convert_utn57_to_zvvnmod(
            "\u{1832}\u{1820}\u{182F}\u{180E}\u{1820}\u{0020}\u{1836}\u{1822}\u{1828}"
        )
        .unwrap()),
        "E042 E005 E03B E00D 0020 E050 E006 E00C"
    );
}

#[test]
fn a_word_space_stays_a_word_space_through_both_directions() {
    let hub = "\u{E042}\u{E005}\u{E03B}\u{E00D}\u{0020}\u{E042}\u{E005}\u{E03B}\u{E00D}";

    assert_eq!(
        hex(&convert_zvvnmod_to_utn57(hub).unwrap()),
        "1832 1820 182F 180E 1820 0020 1832 1820 182F 180E 1820"
    );
}

#[test]
fn the_boundary_is_read_back_as_mvs_and_delimits_the_runs_it_separates() {
    // The hub text above, read back. ᠶᠢᠨ is positioned as its own word, exactly
    // as a detached suffix is, and the boundary returns as MVS.
    let hub = "\u{E042}\u{E005}\u{E03B}\u{E00D}\u{202F}\u{E04D}\u{E006}\u{E00C}";

    assert_eq!(
        hex(&convert_zvvnmod_to_utn57(hub).unwrap()),
        "1832 1820 182F 180E 1820 180E 1822 180B 1822 180D 1820 180C"
    );
}

#[test]
fn zvvnmod_text_carrying_the_old_utn57_control_still_reads_its_boundary_back() {
    // Text this crate emitted before 0.1.2 spelled the boundary U+180E. It is
    // not a ZVVNMOD code, so it passes through — and lands on the same MVS.
    let old = "\u{E042}\u{E005}\u{E03B}\u{E00D}\u{180E}\u{E04D}\u{E006}\u{E00C}";
    let current = "\u{E042}\u{E005}\u{E03B}\u{E00D}\u{202F}\u{E04D}\u{E006}\u{E00C}";

    assert_eq!(
        convert_zvvnmod_to_utn57(old).unwrap(),
        convert_zvvnmod_to_utn57(current).unwrap()
    );
}
