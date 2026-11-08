//! ZWNJ (`U+200C`) is a joining break inside a Mongolian word in `mongol-norm`
//! 0.4. Reverse conversion shapes the whole word so context looks through it
//! and positions each side independently. The break itself is not written into
//! ZVVNMOD: the PUA codes already *are* the unjoined presentation forms.
//! Forward conversion still treats input ZWNJ as passthrough.

use zvvnmod_utn57::{
    convert_utn57_run_to_zvvnmod, convert_utn57_to_zvvnmod, convert_zvvnmod_run,
    convert_zvvnmod_to_utn57, shape_utn57_positioned_written_units, Utn57PositionedWrittenUnit,
    Utn57TextConversionError, AA_FINA, A_FINA, A_INIT, B_A_INIT, B_INIT,
};

fn hex(text: &str) -> String {
    text.chars()
        .map(|character| format!("{:04X}", character as u32))
        .collect::<Vec<_>>()
        .join(" ")
}

fn spell(records: &[Utn57PositionedWrittenUnit]) -> String {
    records
        .iter()
        .map(|record| {
            let position = record.position.contract_name();
            if position == "control" {
                record.written_unit.contract_name().to_owned()
            } else {
                format!("{}:{position}", record.written_unit.contract_name())
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// ᠪ ZWNJ ᠠ — a break that `mongol-norm` 0.4 keeps, because B:fina ≢ B:medi
/// next to A.
const B_ZWNJ_A: &str = "\u{182A}\u{200C}\u{1820}";

#[test]
fn zwnj_splits_chains_without_padding_them_as_a_joiner() {
    assert_eq!(
        spell(&shape_utn57_positioned_written_units(B_ZWNJ_A).unwrap()),
        "B:init A:init A:fina"
    );
}

#[test]
fn a_lone_zwnj_shapes_to_no_written_units() {
    assert_eq!(
        shape_utn57_positioned_written_units("\u{200C}").unwrap(),
        []
    );
}

#[test]
fn reverse_conversion_does_not_write_zwnj_into_zvvnmod() {
    let zvvnmod = convert_utn57_to_zvvnmod(B_ZWNJ_A).unwrap();
    assert!(
        !zvvnmod.contains('\u{200C}'),
        "ZVVNMOD must not carry U+200C: {}",
        hex(&zvvnmod)
    );
    assert_eq!(
        zvvnmod
            .chars()
            .map(zvvnmod_utn57::zvvnmod_code)
            .collect::<Vec<_>>(),
        [Some(B_INIT), Some(A_INIT), Some(A_FINA)]
    );
}

#[test]
fn unjoined_presentation_forms_differ_from_the_joined_word() {
    // Per-segment positioning is what preserves the break's ink: ᠪ‌ᠠ is
    // B:init + isolated A, while ᠪᠠ is the bowed ligature. U+200C is not
    // required to tell those ZVVNMOD spellings apart.
    let broken = convert_utn57_to_zvvnmod(B_ZWNJ_A).unwrap();
    let joined = convert_utn57_to_zvvnmod("\u{182A}\u{1820}").unwrap();
    assert_ne!(
        broken,
        joined,
        "broken {} joined {}",
        hex(&broken),
        hex(&joined)
    );
    assert_eq!(
        joined
            .chars()
            .map(zvvnmod_utn57::zvvnmod_code)
            .collect::<Vec<_>>(),
        [Some(B_A_INIT), Some(AA_FINA)]
    );
}

#[test]
fn positioned_unit_roundtrip_does_not_need_zwnj() {
    let records = shape_utn57_positioned_written_units(B_ZWNJ_A).unwrap();
    let codes = convert_utn57_run_to_zvvnmod(&records).unwrap();
    assert_eq!(codes, [B_INIT, A_INIT, A_FINA]);
    assert_eq!(convert_zvvnmod_run(&codes).unwrap(), records);
}

#[test]
fn complete_text_forward_of_unjoined_glyphs_cannot_rejoin_them() {
    // Concatenated B:init A:init A:fina is valid ZVVNMOD ink but not one
    // Unicode joining chain, so the text-level encoder rejects it rather
    // than inventing a ZWNJ.
    let zvvnmod = convert_utn57_to_zvvnmod(B_ZWNJ_A).unwrap();
    assert!(matches!(
        convert_zvvnmod_to_utn57(&zvvnmod),
        Err(Utn57TextConversionError::Normalize(_))
    ));
}

#[test]
fn input_zwnj_is_passthrough_between_independently_normalized_runs() {
    // B i (U+E029) and A isol (U+E000 U+E00D). The ZWNJ never enters a run.
    let output = convert_zvvnmod_to_utn57("\u{E029}\u{200C}\u{E000}\u{E00D}").unwrap();
    assert!(
        output.contains('\u{200C}'),
        "expected passthrough ZWNJ in {}",
        hex(&output)
    );
    let (left, rest) = output.split_once('\u{200C}').expect("one ZWNJ");
    assert!(!left.is_empty());
    assert!(!rest.contains('\u{200C}'));
}

#[test]
fn a_lone_zwnj_has_no_zvvnmod_presentation() {
    assert_eq!(convert_utn57_to_zvvnmod("\u{200C}").unwrap(), "");
    assert_eq!(convert_zvvnmod_to_utn57("\u{200C}").unwrap(), "\u{200C}");
}
