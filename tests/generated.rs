use zvvnmod_utn57::{
    convert_zvvnmod_run, discard_legacy_controls, replace_ir_fina, zvvnmod_code_decomposition_map,
    IrFinaReplacementError, Utn57Position, Utn57Unit, Utn57WrittenUnit, ZvvnmodCode, AA_FINA,
    A_FINA, A_INIT, A_MEDI, B_INIT, B_I_INIT, HX_AA_FINA, HX_INIT, H_FINA, IR_FINA,
    IR_FINA_REPLACEMENTS, I_FINA, I_ISOL, I_MEDI, L_FINA, M_FINA, N_AA_FINA, O_MEDI, R_FINA,
    S_FINA, T_FINA, T_MEDI, UE_FINA, U_FINA, ZVVNMOD_CODE_DECOMPOSITIONS,
};

fn unit(unit: Utn57Unit, position: Utn57Position) -> Utn57WrittenUnit {
    Utn57WrittenUnit::new(unit, position)
}

#[test]
fn reviewed_chachlag_rules_emit_mvs_by_longest_match() {
    let cases: &[(&[ZvvnmodCode], Utn57Unit)] = &[
        (&[N_AA_FINA], Utn57Unit::N),
        (&[HX_AA_FINA], Utn57Unit::Hx),
        (&[M_FINA, AA_FINA], Utn57Unit::M),
        (&[L_FINA, AA_FINA], Utn57Unit::L),
        (&[S_FINA, AA_FINA], Utn57Unit::S),
        (&[R_FINA, AA_FINA], Utn57Unit::R),
        (&[I_ISOL, AA_FINA], Utn57Unit::I),
        (&[I_FINA, AA_FINA], Utn57Unit::I),
        (&[U_FINA, AA_FINA], Utn57Unit::U),
        (&[H_FINA, AA_FINA], Utn57Unit::H),
    ];
    for &(input, onset) in cases {
        let onset_position = if input == [I_ISOL, AA_FINA] {
            Utn57Position::Isol
        } else {
            Utn57Position::Fina
        };
        assert_eq!(
            convert_zvvnmod_run(input).unwrap(),
            vec![
                unit(onset, onset_position),
                unit(Utn57Unit::MVS, Utn57Position::Control),
                unit(Utn57Unit::Aa, Utn57Position::Isol),
            ],
            "failed source sequence: {input:?}",
        );
    }
}

#[test]
fn standalone_aa_is_isolated_without_mvs() {
    assert_eq!(
        convert_zvvnmod_run(&[AA_FINA]).unwrap(),
        vec![unit(Utn57Unit::Aa, Utn57Position::Isol)],
    );
}

#[test]
fn rejected_inferred_chachlag_sequences_do_not_emit_mvs() {
    for input in [
        &[AA_FINA, AA_FINA][..],
        &[A_FINA, AA_FINA][..],
        &[I_MEDI, AA_FINA, AA_FINA][..],
    ] {
        let output = convert_zvvnmod_run(input).unwrap();
        assert!(
            output.iter().all(|target| target.unit != Utn57Unit::MVS),
            "unexpected MVS for {input:?}",
        );
    }
}

#[test]
fn merged_code_maps_to_decomposed_code_sequence() {
    let map = zvvnmod_code_decomposition_map();
    assert_eq!(map.get(&B_I_INIT), Some(&[B_INIT, I_MEDI].as_slice()));
    assert_eq!(map.len(), 59);
    assert_eq!(map.get(&N_AA_FINA), None);
    for &(merged, components) in ZVVNMOD_CODE_DECOMPOSITIONS {
        assert_eq!(map.get(&merged), Some(&components));
    }
}

#[test]
fn legacy_controls_are_discarded_before_replacements() {
    assert_eq!(
        discard_legacy_controls(&[
            ZvvnmodCode(0xE13F),
            A_INIT,
            ZvvnmodCode(0xE140),
            ZvvnmodCode(0xE141),
            A_MEDI,
            ZvvnmodCode(0xE142),
            ZvvnmodCode(0xE143),
            IR_FINA,
            ZvvnmodCode(0xE144),
        ]),
        vec![
            ZvvnmodCode(0xE13F),
            A_INIT,
            A_MEDI,
            IR_FINA,
            ZvvnmodCode(0xE144),
        ],
    );
}

#[test]
fn hx_codes_keep_their_source_codepoints() {
    assert_eq!(HX_INIT, ZvvnmodCode(0xE034));
    assert_eq!(HX_AA_FINA, ZvvnmodCode(0xE09D));
}

#[test]
fn every_ir_fina_replacement_produces_its_specific_result() {
    assert_eq!(IR_FINA_REPLACEMENTS.len(), 30);
    for &(prefix, result) in IR_FINA_REPLACEMENTS {
        assert_eq!(replace_ir_fina(&[prefix, IR_FINA]), Ok(vec![result]));
    }
}

#[test]
fn ir_fina_replacement_rewrites_a_complete_stream() {
    assert_eq!(
        replace_ir_fina(&[A_INIT, O_MEDI, IR_FINA, T_MEDI, IR_FINA]),
        Ok(vec![A_INIT, UE_FINA, T_FINA]),
    );
}

#[test]
fn unmatched_ir_fina_is_an_error() {
    assert_eq!(
        replace_ir_fina(&[IR_FINA]),
        Err(IrFinaReplacementError {
            index: 0,
            preceding: None,
        }),
    );
    assert_eq!(
        replace_ir_fina(&[A_INIT, IR_FINA]),
        Err(IrFinaReplacementError {
            index: 1,
            preceding: Some(A_INIT),
        }),
    );
    assert_eq!(
        replace_ir_fina(&[O_MEDI, IR_FINA, IR_FINA]),
        Err(IrFinaReplacementError {
            index: 2,
            preceding: Some(UE_FINA),
        }),
    );
}
