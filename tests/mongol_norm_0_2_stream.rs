//! Regression contract for the duplicate-free written-unit stream and canonical
//! control policy (`mng-canonical/5`) provided by `mongol-norm` 0.4.1.

use zvvnmod_utn57::{
    convert_utn57_to_zvvnmod, convert_zvvnmod_to_utn57,
    mongol_norm::{Locale, Shaper, WrittenUnit},
    normalize_positioned_written_units, shape_utn57_positioned_written_units, Utn57Position,
    Utn57PositionedWrittenUnit, Utn57WrittenUnit, AA_FINA, A_FINA, A_INIT, A_MEDI, B_INIT,
    B_I_ISOL, H_FINA, I_MEDI, L_FINA, M_INIT,
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
fn zwnj_positions_adjacent_letters_without_emitting_u200c_in_zvvnmod_and_synthesizes_back() {
    // 1. `ᠠᠯ‌ᠮᠠ` (`A L ZWNJ M A`): ZWNJ is encoded purely in the glyph positions
    //    (`[A:init, A:medi, L:fina, M:init, A:fina]`), emitting `A_INIT A_MEDI L_FINA M_INIT A_FINA`
    //    with no `U+200C` in ZVVNMOD. Converting back to UTN #57 synthesizes the
    //    ZWNJ across the `L:fina` / `M:init` sub-chain boundary (`mongol-norm` 0.4.1).
    let al_zwnj_ma = "\u{1820}\u{182F}\u{200C}\u{182E}\u{1820}";
    let expected_zvvnmod = format!(
        "{}{}{}{}{}",
        A_INIT.as_char().unwrap(),
        A_MEDI.as_char().unwrap(),
        L_FINA.as_char().unwrap(),
        M_INIT.as_char().unwrap(),
        A_FINA.as_char().unwrap(),
    );
    let zvvnmod = convert_utn57_to_zvvnmod(al_zwnj_ma).unwrap();
    assert_eq!(zvvnmod, expected_zvvnmod);
    assert!(!zvvnmod.contains('\u{200C}'));
    assert_eq!(
        convert_zvvnmod_to_utn57(&zvvnmod).unwrap(),
        "\u{1820}\u{180B}\u{1820}\u{182F}\u{200C}\u{182E}\u{1820}\u{180C}"
    );

    // 2. `ᠪ‌ᠠ` (`B ZWNJ A`): `[B:init, A:init, A:fina]` -> `B_INIT A_INIT A_FINA`,
    //    which synthesizes ZWNJ between `B:init` and `A:init` on the forward path.
    let b_zwnj_a = "\u{182A}\u{200C}\u{1820}";
    let b_a_zvvnmod = convert_utn57_to_zvvnmod(b_zwnj_a).unwrap();
    assert_eq!(
        b_a_zvvnmod,
        format!(
            "{}{}{}",
            B_INIT.as_char().unwrap(),
            A_INIT.as_char().unwrap(),
            A_FINA.as_char().unwrap(),
        )
    );
    assert_eq!(
        convert_zvvnmod_to_utn57(&b_a_zvvnmod).unwrap(),
        "\u{182A}\u{200C}\u{1820}\u{180B}\u{1820}\u{180C}"
    );

    // 3. `ᠠᠯ‌ᠠ᠌` (`A L ZWNJ A FVS2`): shapes to `[A:init, A:medi, L:fina, Aa:isol]`,
    //    which maps to `A_INIT A_MEDI L_FINA AA_FINA` without `U+200C` and
    //    matches the forward chachlag rule `L_FINA AA_FINA -> L:fina MVS Aa:isol`
    //    (`ᠠ᠋ᠠᠯ᠎ᠠ`).
    let al_zwnj_aa = "\u{1820}\u{182F}\u{200C}\u{1820}\u{180C}";
    let al_aa_zvvnmod = convert_utn57_to_zvvnmod(al_zwnj_aa).unwrap();
    assert_eq!(
        al_aa_zvvnmod,
        format!(
            "{}{}{}{}",
            A_INIT.as_char().unwrap(),
            A_MEDI.as_char().unwrap(),
            L_FINA.as_char().unwrap(),
            AA_FINA.as_char().unwrap(),
        )
    );
    assert_eq!(
        convert_zvvnmod_to_utn57(&al_aa_zvvnmod).unwrap(),
        "\u{1820}\u{180B}\u{1820}\u{182F}\u{180E}\u{1820}"
    );
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
    let g_fina_zvvnmod = format!(
        "{}{}",
        I_MEDI.as_char().unwrap(),
        AA_FINA.as_char().unwrap()
    );

    assert!(
        masc.contains(H_FINA.as_char().unwrap()),
        "expected masculine H_FINA before ZWNJ, got {masc:?}"
    );
    assert!(
        fem.contains(&g_fina_zvvnmod),
        "expected feminine G:fina (I_MEDI AA_FINA) before ZWNJ, got {fem:?}"
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
    // `[B:init, I:fina]` recomposes to `B_I_ISOL`.
    assert_eq!(
        convert_utn57_to_zvvnmod("\u{182A}\u{200C}\u{1822}\u{180B}").unwrap(),
        B_I_ISOL.as_char().unwrap().to_string()
    );

    // Leading and trailing ZWNJs on a word are always redundant and dropped.
    assert_eq!(
        convert_utn57_to_zvvnmod("\u{200C}\u{182A}\u{1822}\u{180B}\u{200C}").unwrap(),
        B_I_ISOL.as_char().unwrap().to_string()
    );

    // Interior ZWNJ between two `O` units (`ᠠおお‌ᠤ᠋ᠠ` -> `[A, A, O, Zwnj, O, A]`) is
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
    // Consecutive ZWNJs between segments collapse to one break.
    assert_eq!(
        convert_utn57_to_zvvnmod("\u{1820}\u{182F}\u{200C}\u{200C}\u{200C}\u{182E}\u{1820}")
            .unwrap(),
        convert_utn57_to_zvvnmod("\u{1820}\u{182F}\u{200C}\u{182E}\u{1820}").unwrap()
    );

    // A standalone ZWNJ (or repeated ZWNJs) with no letters produces no ZVVNMOD codes.
    assert_eq!(convert_utn57_to_zvvnmod("\u{200C}").unwrap(), "");
    assert_eq!(convert_utn57_to_zvvnmod("\u{200C}\u{200C}").unwrap(), "");

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
