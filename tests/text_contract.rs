use zvvnmod_utn57::{
    classify_zvvnmod_text_character, zvvnmod_code, ZvvnmodTextCharacterKind, ZVVNMOD_CODES,
};

#[test]
fn formal_zvvnmod_shape_inventory_is_exactly_139_codes() {
    assert_eq!(ZVVNMOD_CODES.len(), 139);
    assert!(zvvnmod_code('\u{E001}').is_some());
    assert!(zvvnmod_code('\u{E0E5}').is_some());
    assert_eq!(
        classify_zvvnmod_text_character('\u{E0E5}'),
        ZvvnmodTextCharacterKind::Shape
    );
    assert!(zvvnmod_code('\u{180A}').is_none());
    assert!(zvvnmod_code('\u{E144}').is_none());
    assert!(zvvnmod_code('\u{E23F}').is_none());
}

#[test]
fn character_kind_contract_has_exactly_four_variants() {
    fn label(kind: ZvvnmodTextCharacterKind) -> &'static str {
        match kind {
            ZvvnmodTextCharacterKind::Shape => "shape",
            ZvvnmodTextCharacterKind::LegacyControl => "legacy-control",
            ZvvnmodTextCharacterKind::SuffixSeparator => "suffix-separator",
            ZvvnmodTextCharacterKind::Passthrough => "passthrough",
        }
    }

    assert_eq!(label(ZvvnmodTextCharacterKind::Shape), "shape");
    assert_eq!(
        label(ZvvnmodTextCharacterKind::LegacyControl),
        "legacy-control"
    );
    assert_eq!(
        label(ZvvnmodTextCharacterKind::SuffixSeparator),
        "suffix-separator"
    );
    assert_eq!(label(ZvvnmodTextCharacterKind::Passthrough), "passthrough");
}

#[test]
fn standard_nirugu_mvs_zwj_and_the_word_space_are_passthrough() {
    for character in ['\u{180A}', '\u{180E}', '\u{200D}', '\u{0020}'] {
        assert_eq!(
            classify_zvvnmod_text_character(character),
            ZvvnmodTextCharacterKind::Passthrough,
            "U+{:04X} must pass through",
            character as u32
        );
    }
}

#[test]
fn nnbsp_is_the_detached_suffix_boundary_not_passthrough() {
    assert_eq!(
        classify_zvvnmod_text_character('\u{202F}'),
        ZvvnmodTextCharacterKind::SuffixSeparator
    );
    // It is a boundary, not a shape: it has no code in the formal inventory.
    assert!(zvvnmod_code('\u{202F}').is_none());
}

#[test]
fn legacy_zvvnmod_controls_are_excluded_not_passed_through() {
    for character in ['\u{E140}', '\u{E141}', '\u{E142}', '\u{E143}', '\u{E144}'] {
        assert_eq!(
            classify_zvvnmod_text_character(character),
            ZvvnmodTextCharacterKind::LegacyControl
        );
    }
}

#[test]
fn everything_outside_the_shape_and_control_contract_is_passthrough() {
    let characters = [
        '!',
        '\u{1802}',
        '\u{1810}',
        '\u{2048}',
        '\u{3008}',
        'A',
        '中',
        '😀',
        '\n',
        '\u{E145}',
        '\u{E23F}',
        '\u{E240}',
        '\u{E241}',
        '\u{E242}',
        '\u{F0000}',
        '\u{100000}',
    ];
    for character in characters {
        assert_eq!(
            classify_zvvnmod_text_character(character),
            ZvvnmodTextCharacterKind::Passthrough,
            "U+{:04X} must pass through",
            character as u32
        );
    }
}

#[test]
fn formal_shape_is_still_classified_for_conversion() {
    assert_eq!(
        classify_zvvnmod_text_character('\u{E001}'),
        ZvvnmodTextCharacterKind::Shape
    );
}
