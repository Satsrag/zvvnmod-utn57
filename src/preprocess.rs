use crate::ZvvnmodCode;

/// Discard legacy FVS1/FVS2/FVS3/MVS values from a ZVVNMOD code stream.
///
/// These values are not explicit ZVVNMOD shapes. This preprocessing stage must
/// run before `Ir_fina` replacement, merged-code decomposition, and UTN #57
/// writing-unit mapping.
pub fn discard_legacy_controls(codes: &[ZvvnmodCode]) -> Vec<ZvvnmodCode> {
    codes
        .iter()
        .copied()
        .filter(|code| !matches!(code.0, 0xE140..=0xE143))
        .collect()
}
