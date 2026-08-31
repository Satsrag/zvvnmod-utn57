use std::error::Error;
use zvvnmod_utn57::{convert_zvvnmod_to_utn57, Utn57TextConversionError};

#[test]
fn public_error_boundary_does_not_expose_backend_type_in_its_source_signature() {
    fn assert_error<T: Error>() {}
    assert_error::<Utn57TextConversionError>();
}

#[test]
fn legacy_fvs1_through_fvs4_and_mvs_are_excluded_from_complete_text() {
    assert_eq!(
        convert_zvvnmod_to_utn57("A\u{E140}\u{E141}\u{E142}\u{E143}\u{E144}B").unwrap(),
        "AB"
    );
}

#[test]
fn non_zvvnmod_private_use_characters_pass_through_unchanged() {
    let input = "A\u{E145}\u{E23F}\u{E240}\u{E241}\u{E242}\u{F0000}\u{100000}B";
    assert_eq!(convert_zvvnmod_to_utn57(input).unwrap(), input);
}

#[test]
#[ignore = "requires mongol-norm 0.0.4 installed by zvvnmod-install-mongol-norm"]
fn public_api_converts_complete_mixed_text_in_one_call() {
    let input = "English \u{E001},中\r\n\u{E001}😀";
    let expected = "English \u{1824}\u{180B}\u{200D},中\r\n\u{1824}\u{180B}\u{200D}😀";

    assert_eq!(convert_zvvnmod_to_utn57(input).unwrap(), expected);
}

#[test]
#[ignore = "requires mongol-norm 0.0.4 installed by zvvnmod-install-mongol-norm"]
fn standard_nirugu_passes_through_between_independently_normalized_runs() {
    assert_eq!(
        convert_zvvnmod_to_utn57("\u{E001}\u{180A}\u{E011}").unwrap(),
        "\u{1824}\u{180B}\u{200D}\u{180A}\u{200D}\u{1823}\u{180C}"
    );
}

#[test]
#[ignore = "requires mongol-norm 0.0.4 installed by zvvnmod-install-mongol-norm"]
fn input_zwj_passes_through_between_independently_normalized_runs() {
    // The backend may add a trailing ZWJ to the left run. The input ZWJ is a
    // separate literal and is neither consumed nor deduplicated. The right run
    // remains isolated because input ZWJ has no ZVVNMOD positional semantics.
    assert_eq!(
        convert_zvvnmod_to_utn57("\u{E001}\u{200D}\u{E00D}").unwrap(),
        "\u{1824}\u{180B}\u{200D}\u{200D}\u{1820}\u{180C}"
    );
}

#[test]
#[ignore = "requires mongol-norm 0.0.4 installed by zvvnmod-install-mongol-norm"]
fn standard_mvs_passes_through_between_independently_normalized_runs() {
    assert_eq!(
        convert_zvvnmod_to_utn57("\u{E001}\u{180E}\u{E00D}").unwrap(),
        "\u{1824}\u{180B}\u{200D}\u{180E}\u{1820}\u{180C}"
    );
}
