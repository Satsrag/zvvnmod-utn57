//! Every row of the reverse map must survive `positioned units → ZVVNMOD →
//! positioned units` through this crate's own forward converter.
//!
//! Both ends of that trip live in the shape domain, so it is the correctness
//! property the reverse table actually owes. (Round-tripping back to the
//! original Unicode is not: ZVVNMOD encodes glyph shapes and merges the a/n
//! teeth, o/u and d/t distinctions.)

use std::collections::HashMap;
use zvvnmod_utn57::{
    convert_zvvnmod_run, Utn57PositionedWrittenUnit, ZvvnmodCode, A_INIT, ZVVNMOD_CODES,
    ZVVNMOD_CODE_DECOMPOSITIONS,
};

const MAP: &str = include_str!("../data/utn57-zvvnmod-map.csv");
const CODE_NAMES: &str = include_str!("../src/generated/zvvnmod_codes.rs");

/// `K_INIT` and friends are one ZVVNMOD glyph for two UTN units, so the forward
/// converter resolves them to K by default (`Utn57KVariant`). Those rows cannot
/// round-trip without the caller supplying the variant.
const K2_ROWS: [&str; 3] = ["K2:init", "K2:medi", "K2:fina"];

/// `Dd` is the devsger ᠳ that closes a syllable, and ZVVNMOD writes it as a bowl
/// and a tooth — the same two codes as an `O:medi` vowel followed by a tooth of
/// its own. Both readings occur in real words, so the forward converter picks
/// between them the way `mongol-norm` does (rule III.2e): ᠳ is devsger only
/// behind a vowel, which is what makes it a coda.
///
/// A row on its own has nothing behind it, so these two cannot round-trip
/// standalone — the bowl is read as the vowel it is when nothing precedes it.
/// `the_devsger_rows_round_trip_behind_a_vowel` puts them in the context that
/// makes them devsger and checks them there.
const DEVSGER_ROWS: [&str; 2] = ["Dd:medi", "Dd:fina"];

/// UTN units that ZVVNMOD writes as a sequence of other units: it has no
/// separate glyph for them.
///
/// Six of the nine are unambiguous in practice, and those round-trip through the
/// forward map, which maps each shared spelling back to its composite. Their
/// two-unit reading is legal
/// UTN spelling but not conformant, and it never occurs: counted over 2268
/// shaped samples (this crate's 276-word natural list plus mongol-norm's 1992
/// golden vectors), `A:init Aa:isol`, `A:medi Aa:isol`, `O:medi Aa:isol`,
/// `O:init O:medi`, `I:medi Aa:isol` and `N:medi N:medi` appear zero times,
/// while the composites they collide with appear 5, 137, 3, 6, 122 and 136
/// times. For those the composite is the only reading and the trip back is
/// lossless.
///
/// The other three are genuinely ambiguous — both readings occur in real words:
/// `Dd:medi` against `O:medi A:medi` (17 witnesses, ᠮᠣᠩᠭᠣᠯ), `Dd:fina` against
/// `O:medi A:fina` (51, ᠬᠦᠮᠦᠨ) and `H:medi` against `A:medi A:medi` (85,
/// ᠲᠡᠩᠷᠢ). ZVVNMOD cannot tell those apart, so the trip back has to pick one.
///
/// The two `Dd` rows are picked by context — see [`DEVSGER_ROWS`] — so they
/// round-trip only behind a vowel. `H:medi` is still resolved to the composite
/// wherever its spelling occurs, which is the reading ᠠᠪᠤᠭᠰᠠᠨ wants and the
/// wrong one for ᠲᠡᠩᠷᠢ; nothing here has changed for it.
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
        if K2_ROWS.contains(&row.sources.as_str()) || DEVSGER_ROWS.contains(&row.sources.as_str()) {
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

/// The `Dd` rows in the context that makes them devsger: behind a vowel.
///
/// `A_INIT` spells `A:init`, so the bowl that follows it closes a syllable and
/// the composite is the reading — which is how ᠨᠤᠭᠤᠳ and ᠤᠯᠤᠰᠤᠳ keep their ᠳ.
#[test]
fn the_devsger_rows_round_trip_behind_a_vowel() {
    for row in rows()
        .into_iter()
        .filter(|r| DEVSGER_ROWS.contains(&r.sources.as_str()))
    {
        let mut codes = vec![A_INIT];
        codes.extend_from_slice(&row.codes);
        let units = convert_zvvnmod_run(&recompose(&codes))
            .unwrap_or_else(|error| panic!("{}: {error}", row.sources));
        let spelled = units
            .iter()
            .map(|unit| spell(*unit))
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(
            spelled,
            format!("A:init {}", row.sources),
            "{} should be devsger behind a vowel",
            row.sources
        );
    }
}

/// And with nothing behind it, the same spelling is the bowl vowel it is — the
/// reading ᠮᠣᠩᠭᠣᠯ, ᠬᠦᠮᠦᠨ and ᠮᠣᠳᠣᠨ want, where the bowl sits behind a consonant.
#[test]
fn a_devsger_row_on_its_own_reads_as_a_bowl_vowel_and_a_tooth() {
    for row in rows()
        .into_iter()
        .filter(|r| DEVSGER_ROWS.contains(&r.sources.as_str()))
    {
        let units = convert_zvvnmod_run(&recompose(&row.codes))
            .unwrap_or_else(|error| panic!("{}: {error}", row.sources));
        let spelled = units
            .iter()
            .map(|unit| spell(*unit))
            .collect::<Vec<_>>()
            .join(" ");
        let rival = COMPOSITE_UNITS
            .iter()
            .find(|(unit, _)| *unit == row.sources)
            .map(|(_, pair)| *pair)
            .expect("a devsger row is a composite row");
        assert_eq!(spelled, rival, "{} with nothing behind it", row.sources);
    }
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

/// The seven units with no ZVVNMOD glyph, checked against the font inventory
/// rather than trusted: no code in `data/zvvnmod-unicode-names.csv` names them.
/// `Hx:fina` and `N:fina` appear only inside the merged `Hx f Aa f` (U+E09D) and
/// `N f Aa f` (U+E077), which the chachlag rows already carry.
const WITHOUT_GLYPH: [&str; 7] = [
    "Gx:init", "Gx:medi", "Hx:fina", "Ix:isol", "N:fina", "Sz:fina", "Ux:isol",
];

#[test]
fn the_rows_without_a_zvvnmod_spelling_are_exactly_the_units_without_a_glyph() {
    let spelled: Vec<String> = rows().into_iter().map(|row| row.sources).collect();
    let mut missing: Vec<&str> = MAP
        .lines()
        .skip(2)
        .filter_map(|line| {
            let fields: Vec<&str> = line.split(',').collect();
            fields[2].is_empty().then_some(fields[1])
        })
        .filter(|source| *source != "MVS")
        .collect();
    missing.sort_unstable();
    let mut expected = WITHOUT_GLYPH;
    expected.sort_unstable();
    assert_eq!(
        missing, expected,
        "the set of units with no ZVVNMOD glyph changed"
    );
    for unit in WITHOUT_GLYPH {
        assert!(
            !spelled.contains(&unit.to_owned()),
            "{unit} has no glyph but carries a spelling"
        );
    }
}
