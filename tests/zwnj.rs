//! ZWNJ (`U+200C`) is a joining break inside a Mongolian word in `mongol-norm`
//! 0.4. Forward conversion still treats input ZWNJ as passthrough; reverse
//! conversion shapes the whole word so context looks through the break, then
//! writes surviving ZWNJs back.

use zvvnmod_utn57::{
    convert_utn57_to_zvvnmod, convert_zvvnmod_to_utn57, mongol_norm,
    shape_utn57_positioned_written_units, Utn57PositionedWrittenUnit,
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
fn reverse_conversion_writes_a_surviving_zwnj_back() {
    let zvvnmod = convert_utn57_to_zvvnmod(B_ZWNJ_A).unwrap();
    assert!(
        zvvnmod.contains('\u{200C}'),
        "expected a ZWNJ in {zvvnmod:?} ({})",
        hex(&zvvnmod)
    );
}

#[test]
fn zwnj_round_trips_through_both_conversion_directions() {
    let zvvnmod = convert_utn57_to_zvvnmod(B_ZWNJ_A).unwrap();
    let utn57 = convert_zvvnmod_to_utn57(&zvvnmod).unwrap();
    assert!(
        utn57.contains('\u{200C}'),
        "forward lost the break: {}",
        hex(&utn57)
    );
    assert_eq!(convert_utn57_to_zvvnmod(&utn57).unwrap(), zvvnmod);

    let shaper = mongol_norm::Shaper::new(mongol_norm::Locale::Mng);
    assert_eq!(
        shaper.shape(&utn57).unwrap(),
        shaper.shape(B_ZWNJ_A).unwrap()
    );
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
fn a_lone_zwnj_reads_back_as_itself() {
    assert_eq!(convert_utn57_to_zvvnmod("\u{200C}").unwrap(), "\u{200C}");
    assert_eq!(convert_zvvnmod_to_utn57("\u{200C}").unwrap(), "\u{200C}");
}
