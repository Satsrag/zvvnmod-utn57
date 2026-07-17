use zvvnmod_utn57::{
    replace_ir_fina, zvvnmod_code_decomposition_map, IrFinaReplacementError, ZvvnmodCode, A_INIT,
    B_INIT, B_I_INIT, FVS1, FVS2, FVS3, IR_FINA, IR_FINA_REPLACEMENTS, I_MEDI, MVS, N_AA_FINA,
    O_MEDI, T_FINA, T_MEDI, UE_FINA, ZVVNMOD_CODE_DECOMPOSITIONS,
};

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
fn controls_keep_their_fixed_codepoints() {
    assert_eq!(FVS1, ZvvnmodCode(0xE140));
    assert_eq!(FVS2, ZvvnmodCode(0xE141));
    assert_eq!(FVS3, ZvvnmodCode(0xE142));
    assert_eq!(MVS, ZvvnmodCode(0xE143));
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
