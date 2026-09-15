//! A lone initial glyph: the hub writing `A i` (U+E000) or `I i` (U+E04D) with nothing after it
//! (Satsrag/meco-rust#45).
//!
//! UTN #57 carries `A` and `I` at `isol` as well, under the same bare spelling a lone initial
//! would get, so written bare U+E000 read back as `A:isol` (U+E000 U+E00D) and U+E04D as `I:isol`
//! (U+E01A). mongol-norm 0.2.1 spells a lone `A:init` / `I:init` with a trailing ZWJ, as it
//! already did `O:init`, and each glyph reads back as itself. An isolated consonant *is* its
//! initial written unit, so consonants stay bare.

use zvvnmod_utn57::{
    convert_utn57_to_zvvnmod, convert_zvvnmod_to_utn57, convert_zvvnmod_to_utn57_with_warnings,
    Utn57ConversionWarning, A_INIT,
};

#[test]
fn a_lone_initial_a_or_i_is_spelled_with_a_trailing_zwj() {
    assert_eq!(
        convert_zvvnmod_to_utn57("\u{E000}").unwrap(),
        "\u{1820}\u{180B}\u{200D}"
    );
    assert_eq!(
        convert_zvvnmod_to_utn57("\u{E04D}").unwrap(),
        "\u{1822}\u{180B}\u{200D}"
    );
}

#[test]
fn a_lone_initial_vowel_reads_back_as_the_same_glyph() {
    for hub in ["\u{E000}", "\u{E04D}", "\u{E001}"] {
        let utn57 = convert_zvvnmod_to_utn57(hub).unwrap();
        assert_eq!(
            convert_utn57_to_zvvnmod(&utn57).unwrap(),
            hub,
            "{hub:?} via {utn57:?}"
        );
    }
}

#[test]
fn the_isolated_forms_stay_bare() {
    assert_eq!(
        convert_zvvnmod_to_utn57("\u{E000}\u{E00D}").unwrap(),
        "\u{1820}\u{180B}"
    );
    assert_eq!(
        convert_zvvnmod_to_utn57("\u{E01A}").unwrap(),
        "\u{1822}\u{180B}"
    );
}

#[test]
fn a_lone_initial_consonant_stays_bare() {
    // B i, Ch i, D i: an isolated consonant is its initial written unit, so the joiner would
    // carry nothing — and after D it would even read back as T.
    for (hub, utn57) in [
        ("\u{E029}", "\u{182A}"),
        ("\u{E04A}", "\u{1834}"),
        ("\u{E045}", "\u{1833}"),
    ] {
        assert_eq!(convert_zvvnmod_to_utn57(hub).unwrap(), utn57, "{hub:?}");
    }
}

#[test]
fn the_joiner_is_reported_as_invented() {
    let conversion = convert_zvvnmod_to_utn57_with_warnings("\u{E000}").unwrap();
    assert_eq!(
        conversion.warnings,
        vec![Utn57ConversionWarning::InventedZwj {
            run: 0,
            codes: vec![A_INIT],
            count: 1,
        }]
    );
}
