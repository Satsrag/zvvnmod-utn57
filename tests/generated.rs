use zvvnmod_utn57::{
    convert_zvvnmod_run, discard_legacy_controls, replace_ir_fina, zvvnmod_code_decomposition_map,
    IrFinaReplacementError, Utn57Position, Utn57PositionedWrittenUnit, Utn57WrittenUnit,
    ZvvnmodCode, AA_FINA, A_FINA, A_INIT, A_MEDI, B_INIT, B_I_INIT, HX_AA_FINA, HX_INIT, H_FINA,
    IR_FINA, IR_FINA_REPLACEMENTS, I_FINA, I_ISOL, I_MEDI, L_FINA, M_FINA, N_AA_FINA, O_MEDI,
    R_FINA, S_FINA, T_FINA, T_MEDI, UE_FINA, UTN57_POSITIONED_WRITTEN_UNITS, U_FINA,
    ZVVNMOD_CODE_DECOMPOSITIONS,
};

fn unit(written_unit: Utn57WrittenUnit, position: Utn57Position) -> Utn57PositionedWrittenUnit {
    Utn57PositionedWrittenUnit::new(written_unit, position)
}

#[test]
fn positioned_written_units_expose_stable_python_contract_names() {
    for target in UTN57_POSITIONED_WRITTEN_UNITS {
        assert!(!target.written_unit.contract_name().is_empty());
        assert!(!target.position.contract_name().is_empty());
    }

    assert_eq!(Utn57WrittenUnit::MVS.contract_name(), "Mvs");
    assert_eq!(Utn57WrittenUnit::Nirugu.contract_name(), "Nirugu");
    assert_eq!(Utn57Position::Isol.contract_name(), "isol");
    assert_eq!(Utn57Position::Init.contract_name(), "init");
    assert_eq!(Utn57Position::Medi.contract_name(), "medi");
    assert_eq!(Utn57Position::Fina.contract_name(), "fina");
    assert_eq!(Utn57Position::Control.contract_name(), "control");
}

#[test]
fn reviewed_chachlag_rules_emit_mvs_by_longest_match() {
    let cases: &[(&[ZvvnmodCode], Utn57WrittenUnit)] = &[
        (&[N_AA_FINA], Utn57WrittenUnit::N),
        (&[HX_AA_FINA], Utn57WrittenUnit::Hx),
        (&[M_FINA, AA_FINA], Utn57WrittenUnit::M),
        (&[L_FINA, AA_FINA], Utn57WrittenUnit::L),
        (&[S_FINA, AA_FINA], Utn57WrittenUnit::S),
        (&[R_FINA, AA_FINA], Utn57WrittenUnit::R),
        (&[I_ISOL, AA_FINA], Utn57WrittenUnit::I),
        (&[I_FINA, AA_FINA], Utn57WrittenUnit::I),
        (&[U_FINA, AA_FINA], Utn57WrittenUnit::U),
        (&[H_FINA, AA_FINA], Utn57WrittenUnit::H),
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
                unit(Utn57WrittenUnit::MVS, Utn57Position::Control),
                unit(Utn57WrittenUnit::Aa, Utn57Position::Isol),
            ],
            "failed source sequence: {input:?}",
        );
    }
}

#[test]
fn standalone_aa_is_isolated_without_mvs() {
    assert_eq!(
        convert_zvvnmod_run(&[AA_FINA]).unwrap(),
        vec![unit(Utn57WrittenUnit::Aa, Utn57Position::Isol)],
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
            output
                .iter()
                .all(|target| target.written_unit != Utn57WrittenUnit::MVS),
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
