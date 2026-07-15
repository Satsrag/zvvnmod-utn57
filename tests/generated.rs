use zvvnmod_utn57::{
    shape_to_zvvnmod_map, ZvvnmodCode, ZvvnmodShape, B_I_MEDI, B_I_MEDI_ALT_1, FVS1, FVS2, FVS3,
    MVS,
};

#[test]
fn merged_shape_keeps_all_zvvnmod_aliases() {
    let map = shape_to_zvvnmod_map();
    assert_eq!(map[&ZvvnmodShape::B_I_MEDI], &[B_I_MEDI, B_I_MEDI_ALT_1],);
}

#[test]
fn controls_keep_their_fixed_codepoints() {
    assert_eq!(FVS1, ZvvnmodCode(0xE140));
    assert_eq!(FVS2, ZvvnmodCode(0xE141));
    assert_eq!(FVS3, ZvvnmodCode(0xE142));
    assert_eq!(MVS, ZvvnmodCode(0xE143));
}
