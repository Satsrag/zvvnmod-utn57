//! A ZWJ the normalizer invents is a gap in the hub inventory, not a spelling
//! the source asked for: the run began or ended with a joined-form glyph that
//! ZVVNMOD has no unjoined form for (Satsrag/meco-rust#32). The conversion
//! still succeeds — the ink is right — but the caller is told.

use zvvnmod_utn57::{
    convert_zvvnmod_to_utn57, convert_zvvnmod_to_utn57_with_warnings, Utn57ConversionWarning,
    A_MEDI, B_FINA, B_MEDI, G_O_FINA, G_O_MEDI, I_FINA, O_FINA, O_INIT, U_FINA,
};

#[test]
fn a_run_spelled_with_an_invented_zwj_is_reported() {
    let conversion = convert_zvvnmod_to_utn57_with_warnings("\u{E00F}").unwrap();

    assert_eq!(conversion.text, "\u{200D}\u{1823}\u{180B}");
    assert_eq!(
        conversion.warnings,
        vec![Utn57ConversionWarning::InventedZwj {
            run: 0,
            codes: vec![O_FINA],
            count: 1,
        }]
    );
}

#[test]
fn a_run_the_inventory_can_spell_raises_no_warning() {
    let conversion = convert_zvvnmod_to_utn57_with_warnings("\u{E096} \u{E000}\u{E00C}").unwrap();

    assert!(conversion.warnings.is_empty(), "{:?}", conversion.warnings);
    assert_eq!(
        conversion.text,
        convert_zvvnmod_to_utn57("\u{E096} \u{E000}\u{E00C}").unwrap()
    );
}

#[test]
fn input_zwj_is_passthrough_and_never_counted() {
    assert!(convert_zvvnmod_to_utn57_with_warnings("a\u{200D}b")
        .unwrap()
        .warnings
        .is_empty());
}

#[test]
fn runs_are_numbered_in_input_order_across_passthrough_text() {
    // Run 0 is a lone O:init, which mongol-norm spells with a trailing ZWJ;
    // run 1 (E000 E00C) is a whole word. Only run 0 is reported.
    let conversion = convert_zvvnmod_to_utn57_with_warnings("\u{E001}, \u{E000}\u{E00C}").unwrap();

    assert_eq!(
        conversion.warnings,
        vec![Utn57ConversionWarning::InventedZwj {
            run: 0,
            codes: vec![O_INIT],
            count: 1,
        }]
    );
}

#[test]
fn the_warning_names_the_run_by_its_codes() {
    let conversion = convert_zvvnmod_to_utn57_with_warnings("\u{E00F}").unwrap();
    let message = conversion.warnings[0].to_string();

    assert!(message.contains("U+E00F"), "{message}");
    assert!(message.contains("ZWJ"), "{message}");
}

#[test]
fn the_text_only_entry_point_is_the_same_conversion() {
    for input in [
        "\u{E00F}",
        "\u{E09C}",
        "\u{E096}",
        "\u{E001}\u{202F}\u{E00D}",
        "plain",
    ] {
        assert_eq!(
            convert_zvvnmod_to_utn57(input).unwrap(),
            convert_zvvnmod_to_utn57_with_warnings(input).unwrap().text
        );
    }
}

#[test]
fn lone_glyph_zwj_warnings_follow_mng_canonical_5_equivalence_tables() {
    // Under mongol-norm 0.4.0 (mng-canonical/5), edge ZWJs that produce the
    // same ink as the unjoined form are dropped by `drop_redundant_zwj`, so
    // those runs no longer emit `InventedZwj`:
    // - G_O_FINA (U+E09C, G:medi O:fina): G is in INITIAL_BEFORE_LETTER (G:medi ≡ G:init).
    // - G_O_MEDI (U+E09B, G:medi O:medi): G is in INITIAL_BEFORE_LETTER and O is in FINAL_AFTER_LETTER.
    // - U_FINA (U+E011, U:fina) & I_FINA (U+E00E, I:fina): U and I are in INITIAL_ALONE (fina ≡ isol).
    // - B_MEDI (U+E02A, B:medi): Zwj B Zwj drops leading (B:medi ≡ B:init) then trailing (B:init ≡ B:isol).
    for equivalent_code in [G_O_FINA, G_O_MEDI, U_FINA, I_FINA, B_MEDI] {
        let equivalent_input = equivalent_code.as_char().unwrap().to_string();
        let conversion = convert_zvvnmod_to_utn57_with_warnings(&equivalent_input).unwrap();
        assert!(
            !conversion.text.contains('\u{200D}'),
            "{equivalent_code:?} unexpectedly kept ZWJ: {:?}",
            conversion.text
        );
        assert!(
            conversion.warnings.is_empty(),
            "{equivalent_code:?} unexpectedly warned: {:?}",
            conversion.warnings
        );
    }

    // By contrast, glyphs whose joined and unjoined forms differ keep their
    // invented ZWJ(s) and continue to report `InventedZwj`:
    // - U+E005 (A_MEDI, A:medi): A is in neither INITIAL_BEFORE_LETTER nor FINAL_ALONE (2 ZWJs).
    // - U+E02B (B_FINA, B:fina): lone B is not in INITIAL_ALONE (1 leading ZWJ).
    // - U+E001 (O_INIT, O:init): lone O is not in FINAL_ALONE (1 trailing ZWJ).
    let a_medi = convert_zvvnmod_to_utn57_with_warnings("\u{E005}").unwrap();
    assert_eq!(a_medi.text, "\u{200D}\u{1820}\u{200D}");
    assert_eq!(
        a_medi.warnings,
        vec![Utn57ConversionWarning::InventedZwj {
            run: 0,
            codes: vec![A_MEDI],
            count: 2,
        }]
    );

    let b_fina = convert_zvvnmod_to_utn57_with_warnings("\u{E02B}").unwrap();
    assert_eq!(b_fina.text, "\u{200D}\u{182A}");
    assert_eq!(
        b_fina.warnings,
        vec![Utn57ConversionWarning::InventedZwj {
            run: 0,
            codes: vec![B_FINA],
            count: 1,
        }]
    );
}
