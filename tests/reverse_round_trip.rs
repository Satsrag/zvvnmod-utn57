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

/// Rows the reverse map and the forward map genuinely disagree about, with what
/// the forward converter returns instead. Pinned here so that resolving one
/// fails this test rather than passing silently.
///
/// `Hx:medi`: the reverse map spells medial ɣ `N_MEDI N_MEDI`, which is what
/// meco-core emits and what particle rows 05/32/44 of the forward map use, but
/// the forward map's own `target:Hx:medi` row says `M_MEDI M_MEDI`, so the trip
/// back lands on two N units. Whichever spelling is right, one of the two
/// tables has to change; `M_MEDI M_MEDI` renders as ᠮᠮ through meco-core.
const KNOWN_CONFLICTS: [(&str, &str); 1] = [("Hx:medi", "N:medi N:medi")];

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
        if K2_ROWS.contains(&row.sources.as_str())
            || KNOWN_CONFLICTS
                .iter()
                .any(|(source, _)| *source == row.sources)
        {
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
fn the_known_forward_map_conflicts_are_still_exactly_the_pinned_ones() {
    for (source, expected) in KNOWN_CONFLICTS {
        let row = rows()
            .into_iter()
            .find(|row| row.sources == source)
            .unwrap_or_else(|| panic!("{source} is no longer a row of the reverse map"));
        let units = convert_zvvnmod_run(&recompose(&row.codes)).unwrap();
        let spelled = units
            .iter()
            .map(|unit| spell(*unit))
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(
            spelled, expected,
            "{source} no longer disagrees the way KNOWN_CONFLICTS records; \
             if the forward map was corrected, drop this entry"
        );
    }
}
