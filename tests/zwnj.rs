//! ZWNJ (`U+200C`) is a joining break inside a Mongolian word in `mongol-norm`
//! 0.4. Reverse conversion shapes the whole word so context looks through it,
//! reverse-maps each side independently, and writes surviving breaks back as
//! `U+200C` so a multi-code mapping cannot match across the break. Forward
//! conversion still treats input ZWNJ as passthrough.

use zvvnmod_utn57::{
    convert_utn57_run_to_zvvnmod, convert_utn57_to_zvvnmod, convert_zvvnmod_run,
    convert_zvvnmod_to_utn57, mongol_norm, shape_utn57_positioned_written_units,
    Utn57PositionedWrittenUnit, AA_FINA, A_FINA, A_INIT, A_MEDI, B_A_INIT, B_INIT, L_FINA,
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

fn zvvnmod_codes(text: &str) -> Vec<Option<zvvnmod_utn57::ZvvnmodCode>> {
    text.chars().map(zvvnmod_utn57::zvvnmod_code).collect()
}

/// ᠪ ZWNJ ᠠ — a break that `mongol-norm` 0.4 keeps, because B:fina ≢ B:medi
/// next to A.
const B_ZWNJ_A: &str = "\u{182A}\u{200C}\u{1820}";

/// ᠠᠯ ZWNJ ᠠ FVS2 — left ends `L:fina`, right is `Aa:isol`. Flattened reverse
/// mapping emits `L_FINA AA_FINA`, which forward reads as chachlag / MVS.
const A_L_ZWNJ_AA: &str = "\u{1820}\u{182F}\u{200C}\u{1820}\u{180C}";

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
fn reverse_conversion_writes_a_surviving_zwnj_back_as_a_run_delimiter() {
    let zvvnmod = convert_utn57_to_zvvnmod(B_ZWNJ_A).unwrap();
    assert!(
        zvvnmod.contains('\u{200C}'),
        "expected a ZWNJ delimiter in {}",
        hex(&zvvnmod)
    );
    let (left, right) = zvvnmod.split_once('\u{200C}').expect("one ZWNJ");
    assert_eq!(zvvnmod_codes(left), [Some(B_INIT)]);
    assert_eq!(zvvnmod_codes(right), [Some(A_INIT), Some(A_FINA)]);
}

#[test]
fn unjoined_presentation_forms_differ_from_the_joined_word() {
    let broken = convert_utn57_to_zvvnmod(B_ZWNJ_A).unwrap();
    let joined = convert_utn57_to_zvvnmod("\u{182A}\u{1820}").unwrap();
    assert_ne!(
        broken,
        joined,
        "broken {} joined {}",
        hex(&broken),
        hex(&joined)
    );
    assert_eq!(zvvnmod_codes(&joined), [Some(B_A_INIT), Some(AA_FINA)]);
}

#[test]
fn a_l_zwnj_aa_does_not_become_chachlag() {
    assert_eq!(
        spell(&shape_utn57_positioned_written_units(A_L_ZWNJ_AA).unwrap()),
        "A:init A:medi L:fina Aa:isol"
    );

    let zvvnmod = convert_utn57_to_zvvnmod(A_L_ZWNJ_AA).unwrap();
    assert_eq!(
        zvvnmod_codes(&zvvnmod),
        [
            Some(A_INIT),
            Some(A_MEDI),
            Some(L_FINA),
            None,
            Some(AA_FINA)
        ]
    );
    assert!(
        zvvnmod.contains('\u{200C}'),
        "delimiter required; flattened L_FINA+AA_FINA is chachlag: {}",
        hex(&zvvnmod)
    );

    let utn57 = convert_zvvnmod_to_utn57(&zvvnmod).unwrap();
    assert!(
        utn57.contains('\u{200C}'),
        "forward lost the break: {}",
        hex(&utn57)
    );
    assert!(
        !utn57.contains('\u{180E}'),
        "ZWNJ silently became MVS: {}",
        hex(&utn57)
    );
}

#[test]
fn flattened_run_reverse_of_a_l_zwnj_aa_matches_chachlag() {
    // The public positioned stream omits Zwnj. Reverse-mapping that flattened
    // list as one run is the bug `convert_utn57_to_zvvnmod` must not repeat.
    let records = shape_utn57_positioned_written_units(A_L_ZWNJ_AA).unwrap();
    let codes = convert_utn57_run_to_zvvnmod(&records).unwrap();
    assert_eq!(codes, [A_INIT, A_MEDI, L_FINA, AA_FINA]);
    assert_eq!(
        spell(&convert_zvvnmod_run(&codes).unwrap()),
        "A:init A:medi L:fina Mvs Aa:isol"
    );
}

#[test]
fn b_zwnj_a_positioned_units_round_trip_per_segment() {
    let records = shape_utn57_positioned_written_units(B_ZWNJ_A).unwrap();
    assert_eq!(spell(&records), "B:init A:init A:fina");
    // B‌A happens not to form a multi-code mapping at the junction, so the
    // flattened run reverse is the concatenation of the two segments — unlike
    // A L ZWNJ Aa. Complete-text reverse still inserts U+200C.
    let codes = convert_utn57_run_to_zvvnmod(&records).unwrap();
    assert_eq!(codes, [B_INIT, A_INIT, A_FINA]);
    assert_eq!(convert_zvvnmod_run(&codes).unwrap(), records);
}

#[test]
fn complete_text_forward_does_not_rejoin_across_the_delimiter() {
    let zvvnmod = convert_utn57_to_zvvnmod(B_ZWNJ_A).unwrap();
    let utn57 = convert_zvvnmod_to_utn57(&zvvnmod).unwrap();
    assert!(utn57.contains('\u{200C}'));
    let joined = convert_zvvnmod_to_utn57("\u{E07C}\u{E00D}").unwrap();
    assert_ne!(utn57, joined);
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

#[test]
fn context_looks_through_zwnj_for_vowel_harmony() {
    // G takes the feminine Hx reading when UE is on the other side of the break.
    assert_eq!(
        spell(&shape_utn57_positioned_written_units("\u{182D}\u{200C}\u{1826}").unwrap()),
        "Hx:init A:init Ue:fina"
    );
    assert_eq!(
        spell(&shape_utn57_positioned_written_units("\u{182D}\u{1826}").unwrap()),
        "G:init Ue:fina"
    );
}

#[test]
fn redundant_zwnj_is_dropped() {
    let shaper = mongol_norm::Shaper::new(mongol_norm::Locale::Mng);
    assert_eq!(
        shaper.shape("\u{200C}\u{182A}").unwrap(),
        shaper.shape("\u{182A}").unwrap()
    );
    assert_eq!(
        shaper.shape("\u{182A}\u{200C}").unwrap(),
        shaper.shape("\u{182A}").unwrap()
    );
    use mongol_norm::WrittenUnit::{Zwnj, A, B, I};
    assert_eq!(
        shaper.normalize_written_units(&[B, Zwnj, I]).unwrap(),
        shaper.normalize_written_units(&[B, I]).unwrap()
    );
    assert_ne!(
        shaper.normalize_written_units(&[B, Zwnj, A]).unwrap(),
        shaper.normalize_written_units(&[B, A]).unwrap()
    );
}

#[test]
fn repeated_identical_controls_merge() {
    let shaper = mongol_norm::Shaper::new(mongol_norm::Locale::Mng);
    assert_eq!(
        shaper.shape("\u{182A}\u{200C}\u{200C}\u{1820}").unwrap(),
        shaper.shape(B_ZWNJ_A).unwrap()
    );
    assert_eq!(
        shaper.shape("\u{200D}\u{200D}\u{182A}").unwrap(),
        shaper.shape("\u{200D}\u{182A}").unwrap()
    );
}
