//! Every row of the reverse map must survive `positioned units → ZVVNMOD →
//! positioned units` through this crate's own forward converter.
//!
//! Both ends of that trip live in the shape domain, so it is the correctness
//! property the reverse table actually owes. (Round-tripping back to the
//! original Unicode is not: ZVVNMOD encodes glyph shapes and merges the a/n
//! teeth, o/u and d/t distinctions.)

use std::collections::HashMap;
use zvvnmod_utn57::{
    convert_zvvnmod_run, Utn57PositionedWrittenUnit, ZvvnmodCode, ZVVNMOD_CODES,
    ZVVNMOD_CODE_DECOMPOSITIONS,
};

const MAP: &str = include_str!("../data/utn57-zvvnmod-map.csv");
const CODE_NAMES: &str = include_str!("../src/generated/zvvnmod_codes.rs");

/// `K_INIT` and friends are one ZVVNMOD glyph for two UTN units, so the forward
/// converter resolves them to K by default (`Utn57KVariant`). Those rows cannot
/// round-trip without the caller supplying the variant.
const K2_ROWS: [&str; 3] = ["K2:init", "K2:medi", "K2:fina"];

/// UTN units that ZVVNMOD writes as a sequence of other units: it has no
/// separate glyph for them. Reverse conversion emits the shared spelling and
/// the trip back cannot recover which reading was meant — the same ambiguity
/// forward conversion faces, seen from the other side.
///
/// `every_composite_unit_is_still_ambiguous` derives this list from the table
/// and asserts it is exactly these, so a table change surfaces here.
const COMPOSITE_UNITS: [(&str, &str); 9] = [
    ("A:isol", "A:init Aa:isol"),
    ("Aa:fina", "A:medi Aa:isol"),
    ("B2:fina", "O:medi Aa:isol"),
    ("Cr:init", "O:init O:medi"),
    ("Dd:medi", "O:medi A:medi"),
    ("Dd:fina", "O:medi A:fina"),
    ("G:fina", "I:medi Aa:isol"),
    ("H:medi", "A:medi A:medi"),
    ("Hx:medi", "N:medi N:medi"),
];

/// The one composite whose ZVVNMOD spelling the forward map does not map back
/// to it. ZVVNMOD has no distinct medial ɣ glyph — the shape is exactly
/// `N_MEDI N_MEDI` — but the forward map's `target:Hx:medi` row says
/// `M_MEDI M_MEDI`, so the trip back lands on two N units instead. Pinned so
/// that correcting either table fails this suite rather than passing silently.
const HX_MEDI_RETURNS: &str = "N:medi N:medi";

/// `NAME: ZvvnmodCode = ZvvnmodCode(0x____)` → code point.
fn code_points() -> HashMap<String, u32> {
    let mut map = HashMap::new();
    for line in CODE_NAMES.lines() {
        let Some(rest) = line.strip_prefix("pub const ") else {
            continue;
        };
        let Some((name, tail)) = rest.split_once(':') else {
            continue;
        };
        let Some(hex) = tail
            .split_once("ZvvnmodCode(0x")
            .and_then(|(_, t)| t.split_once(')'))
            .map(|(h, _)| h)
        else {
            continue;
        };
        map.insert(name.to_owned(), u32::from_str_radix(hex, 16).unwrap());
    }
    map
}

/// Merge component runs back into their merged ZVVNMOD glyphs — the fixed
/// runtime step the reverse table is written against.
fn recompose(codes: &[ZvvnmodCode]) -> Vec<ZvvnmodCode> {
    let mut output = Vec::new();
    let mut index = 0;
    while index < codes.len() {
        let merged = ZVVNMOD_CODE_DECOMPOSITIONS.iter().find(|(_, components)| {
            components.len() <= codes.len() - index && codes[index..].starts_with(components)
        });
        match merged {
            Some((merged, components)) => {
                output.push(*merged);
                index += components.len();
            }
            None => {
                output.push(codes[index]);
                index += 1;
            }
        }
    }
    output
}

fn spell(unit: Utn57PositionedWrittenUnit) -> String {
    let position = unit.position.contract_name();
    if position == "control" {
        unit.written_unit.contract_name().to_owned()
    } else {
        format!("{}:{position}", unit.written_unit.contract_name())
    }
}

/// `Mvs` as a control spells `MVS` in the reverse table's `sources` column.
fn normalize(spelling: &str) -> String {
    spelling.replace("Mvs", "MVS")
}

struct Row {
    sources: String,
    codes: Vec<ZvvnmodCode>,
}

fn rows() -> Vec<Row> {
    let points = code_points();
    let mut rows = Vec::new();
    for line in MAP.lines().skip(2) {
        let fields: Vec<&str> = line.split(',').collect();
        assert_eq!(fields.len(), 4, "row must have four fields: {line}");
        if fields[2].is_empty() {
            continue; // no ZVVNMOD glyph for this unit
        }
        let codes: Vec<ZvvnmodCode> = fields[2]
            .split(' ')
            .map(|name| {
                ZvvnmodCode(
                    *points
                        .get(name)
                        .unwrap_or_else(|| panic!("unknown code {name}")),
                )
            })
            .collect();
        rows.push(Row {
            sources: fields[1].to_owned(),
            codes,
        });
    }
    rows
}

#[test]
fn every_reverse_row_names_real_zvvnmod_codes() {
    for row in rows() {
        for code in &row.codes {
            assert!(
                ZVVNMOD_CODES.contains(code),
                "{}: U+{:04X} is outside the formal inventory",
                row.sources,
                code.codepoint()
            );
        }
    }
}

#[test]
fn every_reverse_row_round_trips_through_the_forward_converter() {
    let mut failures = Vec::new();
    for row in rows() {
        if K2_ROWS.contains(&row.sources.as_str()) || row.sources == "Hx:medi" {
            continue;
        }
        let units = convert_zvvnmod_run(&recompose(&row.codes))
            .unwrap_or_else(|error| panic!("{}: {error}", row.sources));
        let spelled = units
            .iter()
            .map(|unit| spell(*unit))
            .collect::<Vec<_>>()
            .join(" ");
        if normalize(&spelled) != normalize(&row.sources) {
            failures.push(format!("  {} → {spelled}", row.sources));
        }
    }
    assert!(
        failures.is_empty(),
        "reverse rows that do not round-trip:\n{}",
        failures.join("\n")
    );
}

#[test]
fn the_k2_rows_are_the_only_ones_the_forward_converter_resolves_to_another_unit() {
    for row in rows()
        .into_iter()
        .filter(|r| K2_ROWS.contains(&r.sources.as_str()))
    {
        let units = convert_zvvnmod_run(&row.codes).unwrap();
        assert_eq!(units.len(), 1);
        assert_eq!(
            spell(units[0]),
            row.sources.replace("K2:", "K:"),
            "{} should resolve to its K counterpart",
            row.sources
        );
    }
}

#[test]
fn hx_medi_still_returns_two_n_units() {
    let row = rows()
        .into_iter()
        .find(|row| row.sources == "Hx:medi")
        .expect("Hx:medi is a row of the reverse map");
    let units = convert_zvvnmod_run(&recompose(&row.codes)).unwrap();
    let spelled = units
        .iter()
        .map(|unit| spell(*unit))
        .collect::<Vec<_>>()
        .join(" ");
    assert_eq!(
        spelled, HX_MEDI_RETURNS,
        "Hx:medi no longer disagrees the way this test records; if the forward map's \
         target:Hx:medi row was corrected to N_MEDI N_MEDI, move Hx:medi back into the \
         round-trip test"
    );
}

/// The composite list is a claim about the data, so derive it and check it.
#[test]
fn every_composite_unit_is_still_ambiguous() {
    let spelled: Vec<(String, Vec<ZvvnmodCode>)> = rows()
        .into_iter()
        .filter(|row| !row.sources.contains(' '))
        .map(|row| (row.sources, recompose(&row.codes)))
        .collect();

    let mut derived = Vec::new();
    for (unit, codes) in &spelled {
        // Which ordered pair of single units spells this unit's glyph sequence?
        for (left, left_codes) in &spelled {
            for (right, right_codes) in &spelled {
                let mut pair = left_codes.clone();
                pair.extend_from_slice(right_codes);
                if &recompose(&pair) == codes && left != unit && right != unit {
                    derived.push((unit.clone(), format!("{left} {right}")));
                }
            }
        }
    }
    derived.sort();
    derived.dedup_by_key(|(unit, _)| unit.clone());

    let expected: Vec<(String, String)> = COMPOSITE_UNITS
        .iter()
        .map(|(unit, pair)| ((*unit).to_owned(), (*pair).to_owned()))
        .collect();
    let mut expected_sorted = expected.clone();
    expected_sorted.sort();
    assert_eq!(
        derived, expected_sorted,
        "the set of UTN units ZVVNMOD writes as a sequence of others changed"
    );
}
