//! A ZWJ the normalizer invents is a gap in the hub inventory, not a spelling
//! the source asked for: the run began or ended with a joined-form glyph that
//! ZVVNMOD has no unjoined form for (Satsrag/meco-rust#32). The conversion
//! still succeeds — the ink is right — but the caller is told.

use zvvnmod_utn57::{
    convert_zvvnmod_to_utn57, convert_zvvnmod_to_utn57_with_warnings, Utn57ConversionWarning,
    G_I_FINA, G_O_FINA, G_O_MEDI, I_FINA, I_ISOL, N_MEDI, O_FINA, O_INIT, P_A_MEDI, P_I_INIT,
    P_O_FINA, U_FINA,
};

#[test]
fn a_run_spelled_with_an_invented_zwj_is_reported() {
    // N:medi still needs joiners on both sides; G:medi O:fina no longer does,
    // because that leading ZWJ is redundant with G:init under mng-canonical/5.
    let conversion = convert_zvvnmod_to_utn57_with_warnings("\u{E028}").unwrap();

    assert_eq!(conversion.text, "\u{200D}\u{1828}\u{180B}\u{200D}");
    assert_eq!(
        conversion.warnings,
        vec![Utn57ConversionWarning::InventedZwj {
            run: 0,
            codes: vec![N_MEDI],
            count: 2,
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
    let conversion = convert_zvvnmod_to_utn57_with_warnings("\u{E028}").unwrap();
    let message = conversion.warnings[0].to_string();

    assert!(message.contains("U+E028"), "{message}");
    assert!(message.contains("ZWJ"), "{message}");
}

#[test]
fn the_text_only_entry_point_is_the_same_conversion() {
    for input in ["\u{E028}", "\u{E096}", "\u{E001}\u{202F}\u{E00D}", "plain"] {
        assert_eq!(
            convert_zvvnmod_to_utn57(input).unwrap(),
            convert_zvvnmod_to_utn57_with_warnings(input).unwrap().text
        );
    }
}

#[test]
fn redundant_zwj_is_not_invented_for_more_lone_glyphs_than_u_fina() {
    // mng-canonical/5 drops a ZWJ whose joined and unjoined ink match. The
    // CHANGELOG's U:fina / G:medi O:fina examples are not the whole set.
    // I_FINA is U+E00E (U+E00F is O_FINA and still invents a leading ZWJ).
    // P_A_MEDI (U+E08A) and P_I_INIT (U+E08C) still invent a trailing ZWJ.
    for code in [I_FINA, U_FINA, G_O_FINA, G_I_FINA, G_O_MEDI, P_O_FINA] {
        let conversion =
            convert_zvvnmod_to_utn57_with_warnings(&code.as_char().unwrap().to_string()).unwrap();
        assert!(
            conversion.warnings.is_empty(),
            "U+{:04X} still invents a ZWJ: {} {:?}",
            code.codepoint(),
            conversion
                .text
                .chars()
                .map(|c| format!("U+{:04X}", c as u32))
                .collect::<Vec<_>>()
                .join(" "),
            conversion.warnings
        );
        assert!(
            !conversion.text.contains('\u{200D}'),
            "U+{:04X} encoded with a ZWJ",
            code.codepoint()
        );
    }
    assert_eq!(
        convert_zvvnmod_to_utn57(&I_FINA.as_char().unwrap().to_string()).unwrap(),
        convert_zvvnmod_to_utn57(&I_ISOL.as_char().unwrap().to_string()).unwrap()
    );

    for code in [O_FINA, P_A_MEDI, P_I_INIT] {
        let conversion =
            convert_zvvnmod_to_utn57_with_warnings(&code.as_char().unwrap().to_string()).unwrap();
        assert!(
            matches!(
                conversion.warnings.as_slice(),
                [Utn57ConversionWarning::InventedZwj { .. }]
            ),
            "U+{:04X} should still invent a ZWJ: {:?}",
            code.codepoint(),
            conversion.warnings
        );
    }
}
