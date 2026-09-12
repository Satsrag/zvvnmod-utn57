//! `G i O f` (U+E096): the word-initial G + O-final ligature the font-derived
//! inventory lacked (Satsrag/meco-rust#32).
//!
//! Every other bowed consonant carries both a `X i O f` and a `X m O f` glyph;
//! G had only `G m O f` (U+E09C), so a whole word such as ᠬᠦ reached the
//! hub as a medial glyph and could only be spelled back with an invented ZWJ.

use zvvnmod_utn57::{
    convert_utn57_to_zvvnmod, convert_zvvnmod_to_utn57, zvvnmod_code_decomposition_map,
    ZvvnmodCode, G_INIT, G_O_FINA, G_O_ISOL, O_FINA, ZVVNMOD_CODES,
};

/// ᠭ FVS2 ᠥ FVS2 — the ink of `G:init O:fina`, pinned the way mongol-norm spells it.
const G_O_ISOL_UTN57: &str = "\u{182D}\u{180C}\u{1825}\u{180C}";

#[test]
fn the_ligature_sits_at_u_e096_and_decomposes_to_its_components() {
    assert_eq!(G_O_ISOL, ZvvnmodCode(0xE096));
    assert!(ZVVNMOD_CODES.contains(&G_O_ISOL));
    assert_eq!(
        zvvnmod_code_decomposition_map().get(&G_O_ISOL),
        Some(&[G_INIT, O_FINA].as_slice())
    );
}

#[test]
fn a_word_initial_g_o_ligature_is_spelled_without_a_zwj() {
    assert_eq!(
        convert_zvvnmod_to_utn57("\u{E096}").unwrap(),
        G_O_ISOL_UTN57
    );
}

#[test]
fn the_medial_g_o_ligature_still_pads_its_open_left_edge() {
    // U+E09C is a medial glyph; alone it still needs the joiner, as any lone
    // medial glyph does. Sources that meant the whole word now write U+E096.
    assert_eq!(
        convert_zvvnmod_to_utn57("\u{E09C}").unwrap(),
        format!("\u{200D}{G_O_ISOL_UTN57}")
    );
    assert_eq!(G_O_FINA, ZvvnmodCode(0xE09C));
}

#[test]
fn the_word_initial_spelling_reads_back_as_the_merged_glyph() {
    assert_eq!(
        convert_utn57_to_zvvnmod(G_O_ISOL_UTN57).unwrap(),
        "\u{E096}"
    );
}

#[test]
fn the_ligature_inside_a_word_still_reads_as_the_medial_glyph() {
    // ᠠᠭᠥ᠌: A:init A:medi G:medi O:fina → A_INIT A_MEDI G_O_FINA, unchanged.
    assert_eq!(
        convert_utn57_to_zvvnmod("\u{1820}\u{182D}\u{1825}\u{180C}").unwrap(),
        "\u{E000}\u{E005}\u{E09C}"
    );
}
