//! A ZWJ the normalizer invents is a gap in the hub inventory, not a spelling
//! the source asked for: the run began or ended with a joined-form glyph that
//! ZVVNMOD has no unjoined form for (Satsrag/meco-rust#32). The conversion
//! still succeeds — the ink is right — but the caller is told.

use zvvnmod_utn57::{
    convert_zvvnmod_to_utn57, convert_zvvnmod_to_utn57_with_warnings, Utn57ConversionWarning,
    G_O_FINA, O_INIT,
};

#[test]
fn a_run_spelled_with_an_invented_zwj_is_reported() {
    let conversion = convert_zvvnmod_to_utn57_with_warnings("\u{E09C}").unwrap();

    assert_eq!(conversion.text, "\u{200D}\u{182D}\u{180C}\u{1825}\u{180C}");
    assert_eq!(
        conversion.warnings,
        vec![Utn57ConversionWarning::InventedZwj {
            run: 0,
            codes: vec![G_O_FINA],
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
    let conversion = convert_zvvnmod_to_utn57_with_warnings("\u{E09C}").unwrap();
    let message = conversion.warnings[0].to_string();

    assert!(message.contains("U+E09C"), "{message}");
    assert!(message.contains("ZWJ"), "{message}");
}

#[test]
fn the_text_only_entry_point_is_the_same_conversion() {
    for input in ["\u{E09C}", "\u{E096}", "\u{E001}\u{202F}\u{E00D}", "plain"] {
        assert_eq!(
            convert_zvvnmod_to_utn57(input).unwrap(),
            convert_zvvnmod_to_utn57_with_warnings(input).unwrap().text
        );
    }
}
