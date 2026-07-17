use zvvnmod_utn57::{
    zvvnmod_code_decomposition_map, ZvvnmodCode, B_INIT, B_I_INIT, FVS1, FVS2, FVS3, I_MEDI, MVS,
    N_AA_FINA, ZVVNMOD_CODE_DECOMPOSITIONS,
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
