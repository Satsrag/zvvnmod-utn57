use zvvnmod_utn57::{
    code_sequence_to_zvvnmod_map, ZvvnmodCode, AA_FINA, B_INIT, B_I_INIT, FVS1, FVS2, FVS3, I_MEDI,
    MVS, N_MEDI, ZVVNMOD_SEQUENCE_REPLACEMENTS,
};

#[test]
fn decomposed_code_sequence_maps_to_merged_code() {
    let map = code_sequence_to_zvvnmod_map();
    assert_eq!(map.get([B_INIT, I_MEDI].as_slice()), Some(&B_I_INIT));
    assert_eq!(map.len(), 59);
    assert_eq!(map.get([N_MEDI, AA_FINA].as_slice()), None);
    for &(sequence, result) in ZVVNMOD_SEQUENCE_REPLACEMENTS {
        assert_eq!(map.get(sequence), Some(&result));
    }
}

#[test]
fn controls_keep_their_fixed_codepoints() {
    assert_eq!(FVS1, ZvvnmodCode(0xE140));
    assert_eq!(FVS2, ZvvnmodCode(0xE141));
    assert_eq!(FVS3, ZvvnmodCode(0xE142));
    assert_eq!(MVS, ZvvnmodCode(0xE143));
}
