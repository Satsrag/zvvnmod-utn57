//! The public reverse-direction surface: `Mongolian text → ZVVNMOD`.

use std::error::Error;
use zvvnmod_utn57::{
    convert_utn57_run_to_zvvnmod, convert_utn57_to_zvvnmod, recompose_zvvnmod_codes,
    shape_utn57_positioned_written_units, Utn57ReverseError, ZvvnmodTextConversionError, A_MEDI,
    B_A_INIT, B_INIT, UTN57_HX_FINA, ZVVNMOD_CODES,
};

/// Spell a converted string as its ZVVNMOD code points.
fn hex(text: &str) -> String {
    text.chars()
        .map(|character| format!("{:04X}", character as u32))
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn the_public_errors_are_standard_errors() {
    fn assert_error<T: Error>() {}
    assert_error::<ZvvnmodTextConversionError>();
    assert_error::<Utn57ReverseError>();
}

#[test]
fn a_word_converts_to_its_zvvnmod_codes() {
    // ᠮᠣᠩᠭᠣᠯ
    assert_eq!(
        hex(&convert_utn57_to_zvvnmod("\u{182E}\u{1823}\u{1829}\u{182D}\u{1823}\u{182F}").unwrap()),
        "E036 E008 E005 E031 E028 E028 E008 E03B"
    );
}

#[test]
fn a_chachlag_suffix_collapses_into_one_merged_code() {
    // ᠰᠢᠨ᠎ᠡ → S_INIT I_MEDI N_AA_FINA
    assert_eq!(
        hex(&convert_utn57_to_zvvnmod("\u{1830}\u{1822}\u{1828}\u{180E}\u{1821}").unwrap()),
        "E03C E006 E077"
    );
}

#[test]
fn a_bowed_consonant_merges_with_the_tooth_that_follows_it() {
    // ᠪᠠ must be the merged B_A_INIT, not the split spelling: downstream
    // converters read the split form as ᠪᠠ᠎ᠠ.
    assert_eq!(
        hex(&convert_utn57_to_zvvnmod("\u{182A}\u{1820}").unwrap()),
        "E07C E00D"
    );
}

#[test]
fn text_outside_mongolian_words_preserves_its_code_points() {
    let input = "English 中 😀\t\r\n\u{1802}\u{1810}";
    assert_eq!(convert_utn57_to_zvvnmod(input).unwrap(), input);
}

#[test]
fn words_and_passthrough_interleave_at_their_original_boundaries() {
    // ᠮᠣᠩᠭᠣᠯ ᠬᠡᠯᠡ᠂ with an ASCII tail.
    let output = convert_utn57_to_zvvnmod(
        "\u{182E}\u{1823}\u{1829}\u{182D}\u{1823}\u{182F} \u{182C}\u{1821}\u{182F}\u{1821}\u{1802} ok",
    )
    .unwrap();

    assert_eq!(
        hex(&output),
        "E036 E008 E005 E031 E028 E028 E008 E03B 0020 E094 E03A E00C 1802 0020 006F 006B"
    );
}

#[test]
fn every_emitted_code_is_in_the_formal_inventory_or_is_a_passed_through_character() {
    // ᠨᠤᠭᠤᠳ ᠤᠨ
    let output = convert_utn57_to_zvvnmod("\u{1828}\u{1824}\u{182D}\u{1824}\u{1833}").unwrap();

    for character in output.chars() {
        let code = zvvnmod_utn57::zvvnmod_code(character);
        assert!(
            code.is_some(),
            "U+{:04X} is outside the formal ZVVNMOD inventory",
            character as u32
        );
    }
    assert!(!ZVVNMOD_CODES.is_empty());
}

#[test]
fn a_unit_without_a_glyph_is_reported_rather_than_substituted() {
    // ᠪᠠᠳᠠᠭ + FVS3 forces a bare Hx:fina, which ZVVNMOD cannot spell.
    let error =
        convert_utn57_to_zvvnmod("\u{182A}\u{1820}\u{1833}\u{1820}\u{182D}\u{180D}").unwrap_err();

    let ZvvnmodTextConversionError::Reverse(reverse) = error else {
        panic!("expected a reverse-stage failure, got {error}");
    };
    assert_eq!(reverse.unit(), UTN57_HX_FINA);
    assert!(reverse.to_string().contains("no ZVVNMOD glyph"));
}

#[test]
fn the_run_level_api_takes_positioned_units_directly() {
    let records = shape_utn57_positioned_written_units("\u{182A}\u{1820}").unwrap();

    assert_eq!(
        convert_utn57_run_to_zvvnmod(&records).unwrap(),
        [B_A_INIT, zvvnmod_utn57::AA_FINA]
    );
}

#[test]
fn recomposition_is_exposed_on_its_own() {
    assert_eq!(recompose_zvvnmod_codes(&[B_INIT, A_MEDI]), [B_A_INIT]);
    assert_eq!(recompose_zvvnmod_codes(&[]), []);
}
