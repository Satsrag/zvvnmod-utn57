use zvvnmod_utn57::{
    convert_zvvnmod_text_with_mongol_norm, normalize_positioned_with_mongol_norm,
    normalize_positioned_with_mongol_norm_python, MongolNormCommandError, Utn57Position, Utn57Unit,
    Utn57WrittenUnit, O_INIT, ZVVNMOD_TO_UTN57_MAPPINGS,
};

#[test]
#[ignore = "requires mongol-norm 0.0.4 installed by zvvnmod-install-mongol-norm"]
fn command_bridge_converts_singleton_o_init() {
    let input = char::from_u32(O_INIT.codepoint()).unwrap().to_string();

    let output = convert_zvvnmod_text_with_mongol_norm(&input).unwrap();

    assert_eq!(output, "\u{1824}\u{180b}\u{200d}");
}

#[test]
#[ignore = "requires mongol-norm 0.0.4 installed by zvvnmod-install-mongol-norm"]
fn command_bridge_normalizes_all_reviewed_target_sequences() {
    for mapping in ZVVNMOD_TO_UTN57_MAPPINGS {
        normalize_positioned_with_mongol_norm(mapping.targets)
            .unwrap_or_else(|error| panic!("{}: {error}", mapping.id));
    }
}

#[test]
#[ignore = "requires mongol-norm 0.0.4 installed by zvvnmod-install-mongol-norm"]
fn command_bridge_preserves_explicit_mvs() {
    let units = [
        Utn57WrittenUnit::new(Utn57Unit::S, Utn57Position::Fina),
        Utn57WrittenUnit::new(Utn57Unit::MVS, Utn57Position::Control),
        Utn57WrittenUnit::new(Utn57Unit::Aa, Utn57Position::Isol),
    ];

    assert_eq!(
        normalize_positioned_with_mongol_norm(&units).unwrap(),
        "\u{200d}\u{1830}\u{180e}\u{1820}"
    );
}

#[test]
fn missing_python_command_returns_a_typed_error() {
    let previous_path = std::env::var_os("ZVVNMOD_MONGOL_NORM_PATH");
    std::env::set_var("ZVVNMOD_MONGOL_NORM_PATH", std::env::temp_dir());
    let error =
        normalize_positioned_with_mongol_norm_python(&[], "/definitely/missing/zvvnmod-python")
            .unwrap_err();
    if let Some(path) = previous_path {
        std::env::set_var("ZVVNMOD_MONGOL_NORM_PATH", path);
    } else {
        std::env::remove_var("ZVVNMOD_MONGOL_NORM_PATH");
    }

    assert!(matches!(error, MongolNormCommandError::Spawn { .. }));
}
