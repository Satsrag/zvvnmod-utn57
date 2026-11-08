//! Regression contract for the duplicate-free written-unit stream and canonical
//! control policy (`mng-canonical/5`) provided by `mongol-norm` 0.4.0.

use zvvnmod_utn57::{
    convert_utn57_to_zvvnmod, convert_zvvnmod_to_utn57,
    mongol_norm::{Locale, Shaper, WrittenUnit},
    normalize_positioned_written_units, shape_utn57_positioned_written_units, Utn57Position,
    Utn57PositionedWrittenUnit, Utn57ReverseError, Utn57WrittenUnit, ZvvnmodTextConversionError,
    AA_FINA, A_INIT, A_MEDI, B_I_ISOL, H_FINA, I_MEDI, L_FINA, M_FINA, R_FINA, S_FINA,
};

fn units(records: &[Utn57PositionedWrittenUnit]) -> Vec<Utn57WrittenUnit> {
    records.iter().map(|record| record.written_unit).collect()
}

fn shape_normalized(raw: &[WrittenUnit]) -> Vec<Utn57WrittenUnit> {
    let shaper = Shaper::new(Locale::Mng);
    let text = shaper.normalize_written_units(raw).unwrap();
    units(&shape_utn57_positioned_written_units(&text).unwrap())
}

#[test]
fn conformant_composite_collisions_are_consumed_as_the_decomposed_stream() {
    use Utn57WrittenUnit::{A as Ua, B as Ub, N as Un, O as Uo, S as Us};
    use WrittenUnit::{Dd, Hx, A, B, H, O, S};

    let cases: &[(&[WrittenUnit], &[Utn57WrittenUnit])] = &[
        (&[A, O, Dd, B, O], &[Ua, Uo, Uo, Ua, Ub, Uo]),
        (&[A, O, Dd], &[Ua, Uo, Uo, Ua]),
        (&[B, A, H, S, A], &[Ub, Ua, Ua, Ua, Us, Ua]),
        (&[B, O, Hx, O, S], &[Ub, Uo, Un, Un, Uo, Us]),
    ];
    for (raw, expected) in cases {
        assert_eq!(shape_normalized(raw), *expected, "raw stream: {raw:?}");
    }
}

#[test]
fn cr_expands_to_o_plus_o_in_all_positions() {
    use Utn57Position::{Fina, Init, Medi};
    use Utn57WrittenUnit::{Cr as Ucr, A as Ua, B as Ub, O as Uo};
    use WrittenUnit::{Cr, A, B};

    let shaper = Shaper::new(Locale::Mng);

    // 1. Isol: lone `ᡂ` (U+1842, `Cr:isol`) and `[Cr]` expand to `[O:init, O:fina]`.
    let isol_expected = vec![
        Utn57PositionedWrittenUnit::new(Uo, Init),
        Utn57PositionedWrittenUnit::new(Uo, Fina),
    ];
    assert_eq!(
        shape_utn57_positioned_written_units("\u{1842}").unwrap(),
        isol_expected
    );
    assert_eq!(
        shape_utn57_positioned_written_units(&shaper.normalize_written_units(&[Cr]).unwrap())
            .unwrap(),
        isol_expected
    );

    // 2. Init: `ᡂᠠ` (U+1842 U+1820), `[Cr, A]`, and `[Cr:init, A:fina]`
    //    expand to `[O:init, O:medi, A:fina]`.
    let init_expected = vec![
        Utn57PositionedWrittenUnit::new(Uo, Init),
        Utn57PositionedWrittenUnit::new(Uo, Medi),
        Utn57PositionedWrittenUnit::new(Ua, Fina),
    ];
    assert_eq!(
        shape_utn57_positioned_written_units("\u{1842}\u{1820}").unwrap(),
        init_expected
    );
    assert_eq!(
        shape_utn57_positioned_written_units(&shaper.normalize_written_units(&[Cr, A]).unwrap())
            .unwrap(),
        init_expected
    );
    assert_eq!(
        shape_utn57_positioned_written_units(
            &normalize_positioned_written_units(&[
                Utn57PositionedWrittenUnit::new(Ucr, Init),
                Utn57PositionedWrittenUnit::new(Ua, Fina),
            ])
            .unwrap()
        )
        .unwrap(),
        init_expected
    );

    // 3. Medi: `ᠪᡂᠠ` (U+182A U+1842 U+1820) and `[B, Cr, A]`
    //    expand to `[B:init, O:medi, O:medi, A:fina]`.
    let medi_expected = vec![
        Utn57PositionedWrittenUnit::new(Ub, Init),
        Utn57PositionedWrittenUnit::new(Uo, Medi),
        Utn57PositionedWrittenUnit::new(Uo, Medi),
        Utn57PositionedWrittenUnit::new(Ua, Fina),
    ];
    assert_eq!(
        shape_utn57_positioned_written_units("\u{182A}\u{1842}\u{1820}").unwrap(),
        medi_expected
    );
    assert_eq!(
        shape_utn57_positioned_written_units(&shaper.normalize_written_units(&[B, Cr, A]).unwrap())
            .unwrap(),
        medi_expected
    );

    // 4. Fina: `ᠪᡂ` (U+182A U+1842) and `[B, Cr]`
    //    expand to `[B:init, O:medi, O:fina]`.
    let fina_expected = vec![
        Utn57PositionedWrittenUnit::new(Ub, Init),
        Utn57PositionedWrittenUnit::new(Uo, Medi),
        Utn57PositionedWrittenUnit::new(Uo, Fina),
    ];
    assert_eq!(
        shape_utn57_positioned_written_units("\u{182A}\u{1842}").unwrap(),
        fina_expected
    );
    assert_eq!(
        shape_utn57_positioned_written_units(&shaper.normalize_written_units(&[B, Cr]).unwrap())
            .unwrap(),
        fina_expected
    );
}

#[test]
fn final_a_aa_contracts_only_immediately_after_a_bowed_written_unit() {
    use Utn57WrittenUnit::{
        Aa as Uaa, Gx as Ugx, A as Ua, B as Ub, F as Uf, G as Ug, K as Uk, K2 as Uk2, N as Un,
        P as Up,
    };
    use WrittenUnit::{Aa, Gx, A, B, F, G, K, K2, N, P};

    for (bowed, expected) in [
        (B, Ub),
        (P, Up),
        (F, Uf),
        (G, Ug),
        (Gx, Ugx),
        (K, Uk),
        (K2, Uk2),
    ] {
        assert_eq!(shape_normalized(&[bowed, A, Aa]), [expected, Uaa]);
    }
    assert_eq!(shape_normalized(&[N, A, Aa]), [Un, Ua, Uaa]);
    assert_eq!(shape_normalized(&[B, A, A, Aa]), [Ub, Ua, Ua, Uaa]);
}

#[test]
fn zwnj_splits_segments_and_a_l_zwnj_a_fvs2_does_not_round_trip_to_mvs() {
    // Regression: ᠠᠯ‌ᠠ᠌ (A L ZWNJ A FVS2, U+1820 U+182F U+200C U+1820 U+180C)
    // shapes to `A:init A:medi L:fina` + ZWNJ + `Aa:isol`.
    // Without emitting U+200C between ZWNJ-split segments, reverse conversion
    // would produce `E000 E005 E03B E00D` (`A_INIT A_MEDI L_FINA AA_FINA`),
    // which matches the forward multi-code rule `L_FINA AA_FINA -> L:fina MVS Aa:isol`
    // across the lost break and corrupts the round-trip into an MVS form (`ᠠᠯ᠎ᠠ`).
    let utn57 = "\u{1820}\u{182F}\u{200C}\u{1820}\u{180C}";
    let expected_zvvnmod = format!(
        "{}{}{}\u{200C}{}",
        A_INIT.as_char().unwrap(),
        A_MEDI.as_char().unwrap(),
        L_FINA.as_char().unwrap(),
        AA_FINA.as_char().unwrap(),
    );

    let zvvnmod = convert_utn57_to_zvvnmod(utn57).unwrap();
    assert_eq!(zvvnmod, expected_zvvnmod);

    let round_tripped = convert_zvvnmod_to_utn57(&zvvnmod).unwrap();
    assert_eq!(
        round_tripped,
        "\u{1820}\u{180B}\u{1820}\u{182F}\u{200C}\u{1820}\u{180C}"
    );
    let shaper = Shaper::new(Locale::Mng);
    assert_eq!(
        shaper.shape(&round_tripped).unwrap(),
        shaper.shape(utn57).unwrap()
    );
    assert!(
        !round_tripped.contains('\u{180E}'),
        "must not introduce MVS: {round_tripped:?}"
    );
    assert_ne!(round_tripped, "\u{1820}\u{182F}\u{180E}\u{1820}");
    assert_ne!(
        round_tripped,
        "\u{1820}\u{180B}\u{1820}\u{182F}\u{180E}\u{1820}"
    );
}

#[test]
fn zwnj_prevents_other_chachlag_multi_code_rules_across_the_break() {
    // Every `X_FINA + AA_FINA -> X:fina MVS Aa:isol` rule must be blocked by
    // the U+200C emitted at a ZWNJ break (`ᠠᠮ‌ᠠ᠌`, `ᠠᠷ‌ᠠ᠌`, `ᠠᠰ‌ᠠ᠌`, `ᠠᠬ‌ᠠ᠌`).
    let shaper = Shaper::new(Locale::Mng);
    for (stem_char, expected_fina) in [
        ('\u{182E}', M_FINA),
        ('\u{1837}', R_FINA),
        ('\u{1830}', S_FINA),
        ('\u{182C}', H_FINA),
    ] {
        let utn57 = format!("\u{1820}{stem_char}\u{200C}\u{1820}\u{180C}");
        let zvvnmod = convert_utn57_to_zvvnmod(&utn57).unwrap();
        assert_eq!(
            zvvnmod,
            format!(
                "{}{}{}\u{200C}{}",
                A_INIT.as_char().unwrap(),
                A_MEDI.as_char().unwrap(),
                expected_fina.as_char().unwrap(),
                AA_FINA.as_char().unwrap(),
            )
        );
        let round_tripped = convert_zvvnmod_to_utn57(&zvvnmod).unwrap();
        assert!(!round_tripped.contains('\u{180E}'), "{round_tripped:?}");
        assert_eq!(
            shaper.shape(&round_tripped).unwrap(),
            shaper.shape(&utn57).unwrap()
        );
    }
}

#[test]
fn context_looks_through_zwnj_for_vowel_harmony() {
    // Backward vowel-harmony propagation (`masc_marker_reaches_g_h`) looks
    // through ZWNJ across the whole word:
    // - In `ᠢᠭ‌ᠯᠠ` (U+1822 U+182D U+200C U+182F U+1820), masculine `ᠠ` after
    //   ZWNJ selects masculine `H:fina` (H_FINA, U+E032) for `ᠭ` before ZWNJ.
    // - In `ᠢᠭ‌ᠯᠡ` (U+1822 U+182D U+200C U+182F U+1821), feminine `ᠡ` after
    //   ZWNJ leaves `ᠭ` before ZWNJ as feminine/default `G:fina` (`I_MEDI AA_FINA`).
    let masc = convert_utn57_to_zvvnmod("\u{1822}\u{182D}\u{200C}\u{182F}\u{1820}").unwrap();
    let fem = convert_utn57_to_zvvnmod("\u{1822}\u{182D}\u{200C}\u{182F}\u{1821}").unwrap();

    let masc_before_zwnj = masc.split('\u{200C}').next().unwrap();
    let fem_before_zwnj = fem.split('\u{200C}').next().unwrap();
    let g_fina_zvvnmod = format!(
        "{}{}",
        I_MEDI.as_char().unwrap(),
        AA_FINA.as_char().unwrap()
    );

    assert!(
        masc_before_zwnj.ends_with(H_FINA.as_char().unwrap()),
        "expected masculine H_FINA before ZWNJ, got {masc_before_zwnj:?}"
    );
    assert!(
        fem_before_zwnj.ends_with(&g_fina_zvvnmod),
        "expected feminine G:fina (I_MEDI AA_FINA) before ZWNJ, got {fem_before_zwnj:?}"
    );

    // Forward vowel-harmony propagation across `ZWNJ + ZWJ` onto a joined
    // second segment (`ᠠᠯ‌‍ᠨᠢᠭ` vs `ᠡᠯ‌‍ᠨᠢᠭ`):
    let forward_masc =
        convert_utn57_to_zvvnmod("\u{1820}\u{182F}\u{200C}\u{200D}\u{1828}\u{1822}\u{182D}")
            .unwrap();
    let forward_fem =
        convert_utn57_to_zvvnmod("\u{1821}\u{182F}\u{200C}\u{200D}\u{1828}\u{1822}\u{182D}")
            .unwrap();

    assert!(forward_masc.ends_with(H_FINA.as_char().unwrap()));
    assert!(forward_fem.ends_with(&g_fina_zvvnmod));
}

#[test]
fn redundant_zwnj_is_dropped_before_reverse_conversion() {
    // `ᠪ‌ᠢ᠋` (B ZWNJ I FVS1): `B:isol ≡ B:init` (`ISOL_TO_INIT`) and `I:isol ≡ I:fina`
    // (`ISOL_TO_FINA`), so the ZWNJ is redundant and dropped; the resulting
    // `[B:init, I:fina]` recomposes to `B_I_ISOL` with no U+200C.
    assert_eq!(
        convert_utn57_to_zvvnmod("\u{182A}\u{200C}\u{1822}\u{180B}").unwrap(),
        B_I_ISOL.as_char().unwrap().to_string()
    );

    // Leading and trailing ZWNJs on a word are always redundant and dropped.
    assert_eq!(
        convert_utn57_to_zvvnmod("\u{200C}\u{182A}\u{1822}\u{180B}\u{200C}").unwrap(),
        B_I_ISOL.as_char().unwrap().to_string()
    );

    // Interior ZWNJ between two `O` units (`ᠠᠥ᠌‌ᠤ᠋ᠠ` -> `[A, A, O, Zwnj, O, A]`) is
    // redundant (`O` is in `FINA_TO_MEDI` and `INIT_TO_MEDI`) and dropped,
    // matching `ᠠᠤᠤᠠ` (`[A, A, O, O, A]`).
    assert_eq!(
        convert_utn57_to_zvvnmod("\u{1820}\u{1825}\u{180C}\u{200C}\u{1824}\u{180B}\u{1820}")
            .unwrap(),
        convert_utn57_to_zvvnmod("\u{1820}\u{1824}\u{1824}\u{1820}").unwrap()
    );

    // ZWNJ adjacent to MVS is redundant and dropped, so chachlag triples still collapse.
    assert_eq!(
        convert_utn57_to_zvvnmod("\u{1830}\u{1822}\u{1828}\u{180E}\u{200C}\u{1821}").unwrap(),
        convert_utn57_to_zvvnmod("\u{1830}\u{1822}\u{1828}\u{180E}\u{1821}").unwrap()
    );
}

#[test]
fn repeated_zwj_and_zwnj_controls_merge() {
    // Consecutive ZWNJs between segments collapse to a single U+200C break.
    assert_eq!(
        convert_utn57_to_zvvnmod("\u{1820}\u{182F}\u{200C}\u{200C}\u{200C}\u{1820}\u{180C}")
            .unwrap(),
        convert_utn57_to_zvvnmod("\u{1820}\u{182F}\u{200C}\u{1820}\u{180C}").unwrap()
    );

    // A standalone run of repeated ZWNJs collapses to one U+200C.
    assert_eq!(
        convert_utn57_to_zvvnmod("\u{200C}\u{200C}").unwrap(),
        "\u{200C}"
    );

    // Repeated ZWJs on a lone initial vowel (`ᠠ᠋‍‍‍`, U+1820 U+180B + 3 ZWJs)
    // merge into one ZWJ and position as a single `A:init` (`A_INIT`).
    assert_eq!(
        convert_utn57_to_zvvnmod("\u{1820}\u{180B}\u{200D}\u{200D}\u{200D}").unwrap(),
        A_INIT.as_char().unwrap().to_string()
    );

    // Repeated interior ZWJs between letters (`ᠯᠢ‍‍ᠡ᠋`) are dropped completely.
    assert_eq!(
        convert_utn57_to_zvvnmod("\u{182F}\u{1822}\u{200D}\u{200D}\u{1821}\u{180B}").unwrap(),
        convert_utn57_to_zvvnmod("\u{182F}\u{1822}\u{1821}\u{180B}").unwrap()
    );
}

#[test]
fn reverse_error_index_is_offset_across_zwnj_segments() {
    // `ᠠᠯ‌ᠪᠠᠳᠠᠭ᠍` (`A L ZWNJ B A D A G+FVS3`): segment 0 (`ᠠᠯ`) has 3 positioned
    // units (`A:init A:medi L:fina`, indices 0..3), and segment 1 (`ᠪᠠᠳᠠᠭ᠍`) has 5
    // (`B:init A:medi D:medi A:medi Hx:fina`, local indices 0..5).
    // `Hx:fina` has no ZVVNMOD glyph and must report index 3 + 4 = 7 in the full
    // word's positioned-unit sequence, not local index 4.
    let err = convert_utn57_to_zvvnmod(
        "\u{1820}\u{182F}\u{200C}\u{182A}\u{1820}\u{1833}\u{1820}\u{182D}\u{180D}",
    )
    .unwrap_err();
    assert_eq!(
        err,
        ZvvnmodTextConversionError::Reverse(Utn57ReverseError::Unrepresentable {
            index: 7,
            unit: Utn57PositionedWrittenUnit::new(Utn57WrittenUnit::Hx, Utn57Position::Fina),
        })
    );
}
