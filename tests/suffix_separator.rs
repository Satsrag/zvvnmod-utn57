//! The detached-suffix boundary, in both directions.
//!
//! ZVVNMOD spells the boundary between a stem and its detached suffix with an
//! ordinary `U+0020`, the same character it writes between two words. UTN #57
//! spells it `MVS`.
//!
//! The baselines below are meco's own hub text — `Delehi -> Zvvnmod`, meco
//! 0.4.0 — character for character, separator included. Of the 49 rows in
//! meco's Java-oracle corpus whose Delehi input carries a `U+202F`, the ZVVNMOD
//! output contains `U+0020` 211 times and `U+202F` not once.
//!
//! Two spellings therefore collapse into one on the hub, and that is a property
//! of the hub rather than a defect here: ZVVNMOD carries the distinction in the
//! *glyph after* the boundary, not in the boundary character. A detached suffix
//! takes a suffix-initial glyph — `E04D` for ᠶᠢᠨ, `E001` for ᠤᠨ — where the same
//! letters opening a word of their own take `E050` and `E000 E008`. Recovering a
//! `MVS` from a `U+0020` means reading those glyphs against a lexical suffix
//! table, which meco has and this crate does not; so the boundary reaches
//! ZVVNMOD intact and does not come back.
//!
//! None of this is testable by round trip. The separator passed through
//! symmetrically in both directions before 0.1.2 and again after it, so
//! `utn57 -> zvvnmod -> utn57` stayed stable across two wrong spellings. Every
//! baseline here is a literal.

use zvvnmod_utn57::{convert_utn57_to_zvvnmod, convert_zvvnmod_to_utn57};

/// Spell a string as its code points.
fn hex(text: &str) -> String {
    text.chars()
        .map(|character| format!("{:04X}", character as u32))
        .collect::<Vec<_>>()
        .join(" ")
}

/// ᠲᠠᠯ᠎ᠠ ᠶᠢᠨ — `tal-a-yin`, a stem ending in a chachlag ᠠ and a consonant-initial
/// detached suffix.
const TAL_A_YIN_HUB: &str = "E042 E005 E03B E00D 0020 E04D E006 E00C";

/// ᠮᠣᠩᠭᠣᠯ ᠤᠨ — `mongol-un`, a vowel-initial detached suffix.
const MONGOL_UN_HUB: &str = "E036 E008 E005 E031 E028 E028 E008 E03B 0020 E001 E00C";

#[test]
fn a_detached_suffix_boundary_is_an_ordinary_space() {
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
fn a_stem_final_chachlag_vowel_carries_its_own_boundary_without_a_separator() {
    // The whole word, so that both of its MVS are in view at once. ᠲᠠᠯ᠎ᠠ ᠶᠢᠨ has
    // two — and exactly one separator, because only one of them is a detached
    // suffix. The other closes the stem: `E00D` is Aa:isol, the detached ᠠ
    // itself, and writing `0020 E00D` in front of it would split ᠲᠠᠯ᠎ᠠ into a
    // word ᠲᠠᠯ and a stray vowel. meco emits no separator there either.
    assert_eq!(TAL_A_YIN_HUB, "E042 E005 E03B E00D 0020 E04D E006 E00C");

    // The stem alone, with nothing after it to separate from.
    // ᠲᠠᠯ᠎ᠠ
    assert_eq!(
        hex(&convert_utn57_to_zvvnmod("\u{1832}\u{1820}\u{182F}\u{180E}\u{1820}").unwrap()),
        "E042 E005 E03B E00D"
    );
    // ᠰᠢᠨ᠎ᠡ, where the chachlag ᠡ merges into the preceding ᠨ as `E077`.
    assert_eq!(
        hex(&convert_utn57_to_zvvnmod("\u{1830}\u{1822}\u{1828}\u{180E}\u{1821}").unwrap()),
        "E03C E006 E077"
    );
}

#[test]
fn a_lone_boundary_converts_in_both_directions() {
    assert_eq!(convert_utn57_to_zvvnmod("\u{180E}").unwrap(), "\u{0020}");
    assert_eq!(convert_utn57_to_zvvnmod("\u{202F}").unwrap(), "\u{0020}");
    assert_eq!(convert_zvvnmod_to_utn57("\u{0020}").unwrap(), "\u{0020}");
}

#[test]
fn a_boundary_before_a_consonant_initial_suffix_survives_on_its_own() {
    // ᠶᠢᠨ behind a boundary, with no stem in front of it.
    assert_eq!(
        hex(&convert_utn57_to_zvvnmod("\u{180E}\u{1836}\u{1822}\u{1828}").unwrap()),
        "0020 E04D E006 E00C"
    );
}

#[test]
fn a_word_space_reaches_the_same_separator_but_a_different_suffix_glyph() {
    // ᠲᠠᠯ᠎ᠠ ᠶᠢᠨ written as two words. The separator is the same `U+0020` — what
    // differs is ᠶ, which takes its word-initial `E050` rather than the
    // suffix-initial `E04D` above. meco's own hub text for the same two words.
    assert_eq!(
        hex(&convert_utn57_to_zvvnmod(
            "\u{1832}\u{1820}\u{182F}\u{180E}\u{1820}\u{0020}\u{1836}\u{1822}\u{1828}"
        )
        .unwrap()),
        "E042 E005 E03B E00D 0020 E050 E006 E00C"
    );

    // ᠮᠠᠯ ᠤᠨ, the same contrast on a vowel-initial suffix: `E001` against the
    // `E000 E008` a word-initial ᠤ is written with.
    assert_eq!(
        hex(&convert_utn57_to_zvvnmod("\u{182E}\u{1820}\u{182F}\u{202F}\u{1824}\u{1828}").unwrap()),
        "E036 E005 E03B 0020 E001 E00C"
    );
    assert_eq!(
        hex(&convert_utn57_to_zvvnmod("\u{182E}\u{1820}\u{182F}\u{0020}\u{1824}\u{1828}").unwrap()),
        "E036 E005 E03B 0020 E000 E008 E00C"
    );
}

#[test]
fn the_hub_text_reads_back_with_the_stem_boundary_intact_and_the_suffix_detached() {
    // ᠲᠠᠯ᠎ᠠ ᠶᠢᠨ on the hub, read back. The stem's own MVS returns, carried by
    // `E00D`. The suffix boundary returns as the `U+0020` the hub spells it
    // with: telling it from a word space needs the lexical suffix table this
    // crate has no counterpart for, so ᠶᠢᠨ reads back as a word of its own.
    let hub = "\u{E042}\u{E005}\u{E03B}\u{E00D}\u{0020}\u{E04D}\u{E006}\u{E00C}";

    assert_eq!(
        hex(&convert_zvvnmod_to_utn57(hub).unwrap()),
        "1832 1820 182F 180E 1820 0020 1822 180B 1822 180D 1820 180C"
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
fn zvvnmod_text_carrying_an_older_boundary_spelling_still_reads_it_back_as_mvs() {
    // 0.1.1 spelled the boundary `U+180E` and 0.1.2 spelled it `U+202F`. Neither
    // is what ZVVNMOD writes, and neither is emitted any more, but text already
    // stored in those spellings still reaches the same MVS.
    let hub_0_1_1 = "\u{E042}\u{E005}\u{E03B}\u{E00D}\u{180E}\u{E04D}\u{E006}\u{E00C}";
    let hub_0_1_2 = "\u{E042}\u{E005}\u{E03B}\u{E00D}\u{202F}\u{E04D}\u{E006}\u{E00C}";

    assert_eq!(
        hex(&convert_zvvnmod_to_utn57(hub_0_1_2).unwrap()),
        "1832 1820 182F 180E 1820 180E 1822 180B 1822 180D 1820 180C"
    );
    assert_eq!(
        convert_zvvnmod_to_utn57(hub_0_1_1).unwrap(),
        convert_zvvnmod_to_utn57(hub_0_1_2).unwrap()
    );
}
