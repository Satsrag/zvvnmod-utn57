use zvvnmod_utn57::{
    convert_zvvnmod_run, convert_zvvnmod_run_with_options, IrFinaReplacementError,
    Utn57ConversionError, Utn57ConversionOptions, Utn57KVariant, Utn57MappingError, Utn57Position,
    Utn57Unit, Utn57WrittenUnit, ZvvnmodCode, AA_FINA, A_FINA, A_INIT, A_MEDI, B_INIT, B_I_INIT,
    CH_MEDI, D_INIT, G_MEDI, HX_AA_FINA, IR_FINA, I_MEDI, K_INIT, NIRUGU, N_AA_FINA, N_INIT,
    N_MEDI, O_INIT, O_MEDI, R_FINA, UE_FINA,
};

#[test]
fn direct_zvvnmod_code_maps_to_typed_utn57_written_unit() {
    assert_eq!(
        convert_zvvnmod_run(&[B_INIT]).unwrap(),
        vec![Utn57WrittenUnit::new(Utn57Unit::B, Utn57Position::Init)],
    );
}

#[test]
fn k_is_the_default_for_the_shared_k_and_k2_shape() {
    assert_eq!(
        convert_zvvnmod_run(&[K_INIT]).unwrap(),
        vec![Utn57WrittenUnit::new(Utn57Unit::K, Utn57Position::Init)],
    );
}

#[test]
fn caller_can_explicitly_select_k2_for_the_shared_shape() {
    let options = Utn57ConversionOptions {
        k_variant: Utn57KVariant::K2,
    };
    assert_eq!(
        convert_zvvnmod_run_with_options(&[K_INIT], options).unwrap(),
        vec![Utn57WrittenUnit::new(Utn57Unit::K2, Utn57Position::Init,)],
    );
}

#[test]
fn aa_fina_is_contextually_isolated_when_it_is_the_complete_run() {
    assert_eq!(
        convert_zvvnmod_run(&[AA_FINA]).unwrap(),
        vec![Utn57WrittenUnit::new(Utn57Unit::Aa, Utn57Position::Isol,)],
    );
}

#[test]
fn merged_code_is_decomposed_before_reviewed_mapping() {
    assert_eq!(
        convert_zvvnmod_run(&[B_I_INIT]).unwrap(),
        convert_zvvnmod_run(&[B_INIT, I_MEDI]).unwrap(),
    );
}

#[test]
fn ir_fina_is_replaced_before_decomposition_and_mapping() {
    assert_eq!(
        convert_zvvnmod_run(&[O_MEDI, IR_FINA]).unwrap(),
        convert_zvvnmod_run(&[UE_FINA]).unwrap(),
    );
}

#[test]
fn legacy_controls_are_discarded_before_all_conversion_stages() {
    assert_eq!(
        convert_zvvnmod_run(&[ZvvnmodCode(0xE140), B_INIT, ZvvnmodCode(0xE143),]).unwrap(),
        convert_zvvnmod_run(&[B_INIT]).unwrap(),
    );
}

#[test]
fn reviewed_multi_code_rule_wins_over_single_code_prefixes() {
    assert_eq!(
        convert_zvvnmod_run(&[A_INIT, AA_FINA]).unwrap(),
        vec![Utn57WrittenUnit::new(Utn57Unit::A, Utn57Position::Isol)],
    );
}

#[test]
fn aa_fina_after_a_connected_form_remains_final() {
    assert_eq!(
        convert_zvvnmod_run(&[B_INIT, AA_FINA]).unwrap(),
        vec![
            Utn57WrittenUnit::new(Utn57Unit::B, Utn57Position::Init),
            Utn57WrittenUnit::new(Utn57Unit::Aa, Utn57Position::Fina),
        ],
    );
}

#[test]
fn retained_chachlag_codes_map_without_invented_decomposition() {
    assert_eq!(
        convert_zvvnmod_run(&[N_AA_FINA]).unwrap(),
        vec![
            Utn57WrittenUnit::new(Utn57Unit::N, Utn57Position::Fina),
            Utn57WrittenUnit::new(Utn57Unit::Aa, Utn57Position::Fina),
        ],
    );
    assert_eq!(
        convert_zvvnmod_run(&[HX_AA_FINA]).unwrap(),
        vec![
            Utn57WrittenUnit::new(Utn57Unit::Hx, Utn57Position::Fina),
            Utn57WrittenUnit::new(Utn57Unit::Aa, Utn57Position::Fina),
        ],
    );
}

#[test]
fn reviewed_particle_corrections_win_over_direct_code_mappings() {
    let u = Utn57WrittenUnit::new;
    let cases = [
        (
            vec![A_INIT, CH_MEDI, A_MEDI, N_MEDI, N_MEDI, A_MEDI, A_FINA],
            vec![
                u(Utn57Unit::A, Utn57Position::Init),
                u(Utn57Unit::Ch, Utn57Position::Medi),
                u(Utn57Unit::A, Utn57Position::Medi),
                u(Utn57Unit::Hx, Utn57Position::Medi),
                u(Utn57Unit::A, Utn57Position::Medi),
                u(Utn57Unit::A, Utn57Position::Fina),
            ],
        ),
        (
            vec![O_INIT, O_MEDI, A_FINA],
            vec![
                u(Utn57Unit::O, Utn57Position::Init),
                u(Utn57Unit::Dd, Utn57Position::Fina),
            ],
        ),
        (
            vec![N_INIT, O_MEDI, G_MEDI, O_MEDI, A_FINA],
            vec![
                u(Utn57Unit::N, Utn57Position::Init),
                u(Utn57Unit::O, Utn57Position::Medi),
                u(Utn57Unit::G, Utn57Position::Medi),
                u(Utn57Unit::O, Utn57Position::Medi),
                u(Utn57Unit::Dd, Utn57Position::Fina),
            ],
        ),
        (
            vec![D_INIT, A_MEDI, N_MEDI, N_MEDI, A_MEDI, A_FINA],
            vec![
                u(Utn57Unit::D, Utn57Position::Init),
                u(Utn57Unit::A, Utn57Position::Medi),
                u(Utn57Unit::Hx, Utn57Position::Medi),
                u(Utn57Unit::A, Utn57Position::Medi),
                u(Utn57Unit::A, Utn57Position::Fina),
            ],
        ),
        (
            vec![D_INIT, A_MEDI, I_MEDI, AA_FINA],
            vec![
                u(Utn57Unit::D, Utn57Position::Init),
                u(Utn57Unit::A, Utn57Position::Medi),
                u(Utn57Unit::G, Utn57Position::Fina),
            ],
        ),
        (
            vec![D_INIT, O_MEDI, N_MEDI, N_MEDI, A_MEDI, R_FINA],
            vec![
                u(Utn57Unit::D, Utn57Position::Init),
                u(Utn57Unit::O, Utn57Position::Medi),
                u(Utn57Unit::Hx, Utn57Position::Medi),
                u(Utn57Unit::A, Utn57Position::Medi),
                u(Utn57Unit::R, Utn57Position::Fina),
            ],
        ),
    ];
    for (sources, expected) in cases {
        assert_eq!(convert_zvvnmod_run(&sources).unwrap(), expected);
    }
}

#[test]
fn nirugu_maps_to_a_non_positional_utn57_control() {
    assert_eq!(
        convert_zvvnmod_run(&[NIRUGU]).unwrap(),
        vec![Utn57WrittenUnit::new(
            Utn57Unit::Nirugu,
            Utn57Position::Control,
        )],
    );
}

#[test]
fn errors_preserve_the_failing_conversion_stage() {
    assert_eq!(
        convert_zvvnmod_run(&[IR_FINA]),
        Err(Utn57ConversionError::IrFina(IrFinaReplacementError {
            index: 0,
            preceding: None,
        })),
    );
    let unknown = ZvvnmodCode(0x10FFFF);
    assert_eq!(
        convert_zvvnmod_run(&[unknown]),
        Err(Utn57ConversionError::Mapping(Utn57MappingError {
            index: 0,
            code: unknown,
        })),
    );
}
