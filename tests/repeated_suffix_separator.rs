//! A detached-suffix boundary written with more than one separator
//! (Satsrag/meco-rust#40).
//!
//! ZVVNMOD spells the boundary as `U+202F`; UTN #57 as one `MVS`, which can
//! stand only once between the stem's final letter and the suffix's first. Two
//! separators in a row still mark one boundary — there is nothing between them
//! to separate — so one MVS is written and the repetition is reported.

use zvvnmod_utn57::{
    convert_utn57_to_zvvnmod, convert_zvvnmod_to_utn57, convert_zvvnmod_to_utn57_with_warnings,
    Utn57ConversionWarning,
};

/// ᠤᠯᠤᠰ ᠤᠨ as meco's hub spells it, with the boundary written `N` times.
fn ulus_un(separators: usize) -> String {
    format!(
        "\u{E000}\u{E008}\u{E03A}\u{E008}\u{E03E}{}\u{E001}\u{E00C}",
        "\u{202F}".repeat(separators)
    )
}

#[test]
fn a_boundary_written_twice_is_one_mvs() {
    assert_eq!(
        convert_zvvnmod_to_utn57(&ulus_un(2)).unwrap(),
        convert_zvvnmod_to_utn57(&ulus_un(1)).unwrap()
    );
    assert_eq!(
        convert_zvvnmod_to_utn57(&ulus_un(2))
            .unwrap()
            .matches('\u{180E}')
            .count(),
        1
    );
}

#[test]
fn the_repetition_is_reported_with_its_count() {
    for separators in [2, 3] {
        let conversion = convert_zvvnmod_to_utn57_with_warnings(&ulus_un(separators)).unwrap();
        assert_eq!(
            conversion.warnings,
            vec![Utn57ConversionWarning::CollapsedSuffixSeparators {
                boundary: 0,
                separators,
            }],
            "{separators} separators"
        );
        let message = conversion.warnings[0].to_string();
        assert!(message.contains(&separators.to_string()), "{message}");
        assert!(message.contains("MVS"), "{message}");
    }
}

#[test]
fn a_boundary_written_once_raises_no_warning() {
    for input in [
        ulus_un(1),
        "\u{202F}".to_owned(),
        "\u{202F}\u{E04D}\u{E006}\u{E00C}".to_owned(),
    ] {
        let conversion = convert_zvvnmod_to_utn57_with_warnings(&input).unwrap();
        assert!(
            conversion.warnings.is_empty(),
            "{input:?}: {:?}",
            conversion.warnings
        );
    }
}

#[test]
fn boundaries_are_numbered_in_input_order() {
    // Two boundaries; only the second is repeated.
    let input = format!("{} {}", ulus_un(1), ulus_un(2));
    let conversion = convert_zvvnmod_to_utn57_with_warnings(&input).unwrap();

    assert_eq!(conversion.text.matches('\u{180E}').count(), 2);
    assert_eq!(
        conversion.warnings,
        vec![Utn57ConversionWarning::CollapsedSuffixSeparators {
            boundary: 1,
            separators: 2,
        }]
    );
}

#[test]
fn the_reverse_direction_still_reads_its_input_faithfully() {
    // Two MVS in UTN #57 input are not valid, but they are what was written.
    assert_eq!(
        convert_utn57_to_zvvnmod(
            "\u{1824}\u{182F}\u{1824}\u{1830}\u{180E}\u{180E}\u{1824}\u{1828}"
        )
        .unwrap(),
        ulus_un(2)
    );
}
