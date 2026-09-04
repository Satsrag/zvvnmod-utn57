use crate::{
    Utn57PositionedWrittenUnit, ZvvnmodCode, UTN57_TO_ZVVNMOD_MAPPINGS, ZVVNMOD_CODE_DECOMPOSITIONS,
};
use std::error::Error;
use std::fmt;

/// The longest source sequence in the reverse relation: the chachlag triples.
const LONGEST_RULE: usize = 3;

/// How ZVVNMOD spells a detached-suffix boundary: `U+202F NARROW NO-BREAK SPACE`.
///
/// A boundary before a chachlag ᠠ/ᠡ is carried by the merged chachlag glyph, so
/// the ten `X:fina MVS Aa:isol` rules consume their `MVS` and emit no separator.
/// Every other `MVS` record is a boundary of its own, and ZVVNMOD keeps it
/// distinct from the `U+0020` it writes between words — which is what lets
/// [`convert_zvvnmod_to_utn57`] read the boundary back as `MVS` without having to
/// guess which spaces are suffix boundaries.
///
/// `U+180E MONGOLIAN VOWEL SEPARATOR` is the UTN #57 spelling of the boundary,
/// not the ZVVNMOD one. Emitting it verbatim leaks a UTN #57 code point into
/// ZVVNMOD text, which every downstream spoke then renders as a stray control.
///
/// [`convert_zvvnmod_to_utn57`]: crate::convert_zvvnmod_to_utn57
pub const ZVVNMOD_SUFFIX_SEPARATOR: ZvvnmodCode = ZvvnmodCode(0x202F);

/// Failure while converting positioned UTN #57 written units to ZVVNMOD.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Utn57ReverseError {
    /// The ZVVNMOD font has no glyph for this unit.
    ///
    /// Reverse conversion reports it rather than substituting a near glyph,
    /// which silently changes the text: encoding a bare `N:fina` as the chachlag
    /// merged code reads back with a vowel that was not in the input.
    Unrepresentable {
        /// Position of the offending record in the request.
        index: usize,
        /// The record with no ZVVNMOD spelling.
        unit: Utn57PositionedWrittenUnit,
    },
    /// The `(unit, position)` pair is outside the reviewed UTN #57 inventory.
    UnknownUnit {
        /// Position of the offending record in the request.
        index: usize,
        /// The record that no relation row covers.
        unit: Utn57PositionedWrittenUnit,
    },
}

impl Utn57ReverseError {
    /// The offending record's position in the request.
    pub const fn index(self) -> usize {
        match self {
            Self::Unrepresentable { index, .. } | Self::UnknownUnit { index, .. } => index,
        }
    }

    /// The offending record.
    pub const fn unit(self) -> Utn57PositionedWrittenUnit {
        match self {
            Self::Unrepresentable { unit, .. } | Self::UnknownUnit { unit, .. } => unit,
        }
    }
}

impl fmt::Display for Utn57ReverseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let unit = self.unit();
        let (unit_name, position) = (
            unit.written_unit.contract_name(),
            unit.position.contract_name(),
        );
        match self {
            Self::Unrepresentable { index, .. } => write!(
                formatter,
                "no ZVVNMOD glyph for {unit_name}:{position} at index {index}"
            ),
            Self::UnknownUnit { index, .. } => write!(
                formatter,
                "{unit_name}:{position} at index {index} is outside the reviewed UTN #57 inventory"
            ),
        }
    }
}

impl Error for Utn57ReverseError {}

/// Merge component ZVVNMOD codes back into their merged glyphs.
///
/// The reviewed reverse relation spells every unit in component form, so this
/// step is what turns `B_INIT A_MEDI` into `B_A_INIT`. It is not cosmetic:
/// downstream converters key on the merged codes, and the split spelling picks
/// up a stray MVS — `B_MEDI A_MEDI AA_FINA` renders as ᠪᠠ᠎ᠠ where
/// `B_A_MEDI AA_FINA` renders as ᠪᠠ.
///
/// Longest match wins, so a three-code decomposition is preferred over a
/// two-code one starting at the same offset.
pub fn recompose_zvvnmod_codes(codes: &[ZvvnmodCode]) -> Vec<ZvvnmodCode> {
    let mut output = Vec::with_capacity(codes.len());
    let mut index = 0;
    while index < codes.len() {
        let merged = ZVVNMOD_CODE_DECOMPOSITIONS
            .iter()
            .filter(|(_, components)| codes[index..].starts_with(components))
            .max_by_key(|(_, components)| components.len());
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

/// Convert one run of positioned UTN #57 written units to ZVVNMOD codes.
///
/// Reviewed longest-match replacement resolves the chachlag triples
/// (`X:fina MVS Aa:isol` → one merged code) before the single-unit rows, then
/// [`recompose_zvvnmod_codes`] merges component runs into their merged glyphs.
/// A `MVS` record no chachlag rule consumed is emitted as
/// [`ZVVNMOD_SUFFIX_SEPARATOR`], the `U+202F` ZVVNMOD writes a detached-suffix
/// boundary with.
///
/// # Errors
///
/// Returns [`Utn57ReverseError::Unrepresentable`] for a unit the ZVVNMOD font
/// has no glyph for, and [`Utn57ReverseError::UnknownUnit`] for a
/// `(unit, position)` pair outside the reviewed inventory. Both carry the
/// offending record's index.
pub fn convert_utn57_run_to_zvvnmod(
    units: &[Utn57PositionedWrittenUnit],
) -> Result<Vec<ZvvnmodCode>, Utn57ReverseError> {
    let mut output = Vec::with_capacity(units.len());
    let mut index = 0;
    while index < units.len() {
        let matched = UTN57_TO_ZVVNMOD_MAPPINGS
            .iter()
            .filter(|rule| units[index..].starts_with(rule.sources))
            .max_by_key(|rule| rule.sources.len());
        let Some(rule) = matched else {
            return Err(Utn57ReverseError::UnknownUnit {
                index,
                unit: units[index],
            });
        };
        debug_assert!(rule.sources.len() <= LONGEST_RULE);
        if rule.targets.is_empty() {
            // MVS is structural, not a missing glyph: spell the boundary.
            if units[index] == crate::UTN57_MVS_CONTROL {
                output.push(ZVVNMOD_SUFFIX_SEPARATOR);
                index += 1;
                continue;
            }
            return Err(Utn57ReverseError::Unrepresentable {
                index,
                unit: units[index],
            });
        }
        output.extend_from_slice(rule.targets);
        index += rule.sources.len();
    }
    Ok(recompose_zvvnmod_codes(&output))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Utn57Position, Utn57WrittenUnit, B_A_INIT, B_A_MEDI, HX_AA_FINA, UTN57_AA_ISOL,
        UTN57_A_MEDI, UTN57_B_INIT, UTN57_B_MEDI, UTN57_HX_FINA, UTN57_MVS_CONTROL, UTN57_N_FINA,
    };

    #[test]
    fn an_empty_run_converts_to_nothing() {
        assert_eq!(convert_utn57_run_to_zvvnmod(&[]).unwrap(), []);
    }

    #[test]
    fn a_component_pair_is_recomposed_into_its_merged_glyph() {
        // B:init + Aa:fina spells B_INIT A_MEDI AA_FINA, which merges to B_A_INIT.
        let units = [UTN57_B_INIT, crate::UTN57_AA_FINA];

        assert_eq!(
            convert_utn57_run_to_zvvnmod(&units).unwrap(),
            [B_A_INIT, crate::AA_FINA]
        );
    }

    #[test]
    fn the_medial_pair_merges_the_same_way() {
        let units = [UTN57_B_MEDI, crate::UTN57_AA_FINA];

        assert_eq!(
            convert_utn57_run_to_zvvnmod(&units).unwrap(),
            [B_A_MEDI, crate::AA_FINA]
        );
    }

    #[test]
    fn a_chachlag_triple_wins_over_its_single_unit_rows() {
        let units = [UTN57_HX_FINA, UTN57_MVS_CONTROL, UTN57_AA_ISOL];

        assert_eq!(convert_utn57_run_to_zvvnmod(&units).unwrap(), [HX_AA_FINA]);
    }

    #[test]
    fn a_unit_without_a_glyph_reports_its_index() {
        let units = [UTN57_B_INIT, UTN57_A_MEDI, UTN57_HX_FINA];

        assert_eq!(
            convert_utn57_run_to_zvvnmod(&units).unwrap_err(),
            Utn57ReverseError::Unrepresentable {
                index: 2,
                unit: UTN57_HX_FINA,
            }
        );
    }

    #[test]
    fn the_same_unit_converts_inside_its_chachlag_context() {
        let units = [UTN57_N_FINA, UTN57_MVS_CONTROL, UTN57_AA_ISOL];

        assert_eq!(
            convert_utn57_run_to_zvvnmod(&units).unwrap(),
            [crate::N_AA_FINA]
        );
    }

    #[test]
    fn a_lone_mvs_becomes_the_zvvnmod_suffix_separator() {
        let units = [UTN57_B_INIT, UTN57_MVS_CONTROL, UTN57_A_MEDI];

        assert_eq!(
            convert_utn57_run_to_zvvnmod(&units).unwrap(),
            [crate::B_INIT, ZVVNMOD_SUFFIX_SEPARATOR, crate::A_MEDI]
        );
    }

    /// `Utn57PositionedWrittenUnit` is a public struct literal, so a caller can
    /// build a pair the reviewed inventory does not contain.
    #[test]
    fn a_pair_outside_the_reviewed_inventory_is_rejected() {
        let unit = Utn57PositionedWrittenUnit::new(Utn57WrittenUnit::B, Utn57Position::Control);

        assert_eq!(
            convert_utn57_run_to_zvvnmod(&[UTN57_B_INIT, unit]).unwrap_err(),
            Utn57ReverseError::UnknownUnit { index: 1, unit }
        );
    }

    #[test]
    fn recomposition_prefers_the_longest_decomposition() {
        // G_O_I_INIT decomposes into three codes; the two-code G_O_MEDI must not
        // win at the same offset.
        let codes = [crate::G_INIT, crate::O_MEDI, crate::I_MEDI];

        assert_eq!(recompose_zvvnmod_codes(&codes), [crate::G_O_I_INIT]);
    }
}
