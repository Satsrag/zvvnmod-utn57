use crate::{
    convert_zvvnmod_run, Utn57ConversionError, Utn57Position, Utn57PositionedWrittenUnit,
    Utn57WrittenUnit, ZvvnmodCode, UTN57_POSITIONED_WRITTEN_UNITS,
};
use mongol_norm::{Error, Locale, PositionedWrittenUnit, Shaper, UnitPosition, WrittenUnit};
use std::sync::OnceLock;

/// The shared Hudum (`MNG`) shaper.
///
/// `Shaper::new` derives its lookup maps from `&'static` tables and the shaper
/// is immutable afterwards, so one instance serves the whole process.
fn shaper() -> &'static Shaper {
    static SHAPER: OnceLock<Shaper> = OnceLock::new();
    SHAPER.get_or_init(|| Shaper::new(Locale::Mng))
}

/// Map a reviewed UTN #57 unit identity onto its `mongol-norm` counterpart.
///
/// The mapping is an exhaustive `match` rather than a lookup through
/// [`Utn57WrittenUnit::contract_name`]: `utn57_mapping.rs` is generated, so a
/// unit added by the generator has to fail this crate's build instead of only
/// failing at run time.
const fn written_unit(unit: Utn57WrittenUnit) -> WrittenUnit {
    match unit {
        Utn57WrittenUnit::A => WrittenUnit::A,
        Utn57WrittenUnit::Aa => WrittenUnit::Aa,
        Utn57WrittenUnit::B => WrittenUnit::B,
        Utn57WrittenUnit::B2 => WrittenUnit::B2,
        Utn57WrittenUnit::C => WrittenUnit::C,
        Utn57WrittenUnit::Ch => WrittenUnit::Ch,
        Utn57WrittenUnit::Cr => WrittenUnit::Cr,
        Utn57WrittenUnit::D => WrittenUnit::D,
        Utn57WrittenUnit::Dd => WrittenUnit::Dd,
        Utn57WrittenUnit::F => WrittenUnit::F,
        Utn57WrittenUnit::G => WrittenUnit::G,
        Utn57WrittenUnit::Gx => WrittenUnit::Gx,
        Utn57WrittenUnit::H => WrittenUnit::H,
        Utn57WrittenUnit::Hr => WrittenUnit::Hr,
        Utn57WrittenUnit::Hx => WrittenUnit::Hx,
        Utn57WrittenUnit::I => WrittenUnit::I,
        Utn57WrittenUnit::Ix => WrittenUnit::Ix,
        Utn57WrittenUnit::J => WrittenUnit::J,
        Utn57WrittenUnit::K => WrittenUnit::K,
        Utn57WrittenUnit::K2 => WrittenUnit::K2,
        Utn57WrittenUnit::L => WrittenUnit::L,
        Utn57WrittenUnit::M => WrittenUnit::M,
        Utn57WrittenUnit::N => WrittenUnit::N,
        Utn57WrittenUnit::O => WrittenUnit::O,
        Utn57WrittenUnit::P => WrittenUnit::P,
        Utn57WrittenUnit::R => WrittenUnit::R,
        Utn57WrittenUnit::Rh => WrittenUnit::Rh,
        Utn57WrittenUnit::S => WrittenUnit::S,
        Utn57WrittenUnit::Sh => WrittenUnit::Sh,
        Utn57WrittenUnit::Sz => WrittenUnit::Sz,
        Utn57WrittenUnit::T => WrittenUnit::T,
        Utn57WrittenUnit::U => WrittenUnit::U,
        Utn57WrittenUnit::Ue => WrittenUnit::Ue,
        Utn57WrittenUnit::Ux => WrittenUnit::Ux,
        Utn57WrittenUnit::W => WrittenUnit::W,
        Utn57WrittenUnit::Y => WrittenUnit::Y,
        Utn57WrittenUnit::Z => WrittenUnit::Z,
        Utn57WrittenUnit::Zr => WrittenUnit::Zr,
        Utn57WrittenUnit::Nirugu => WrittenUnit::Nirugu,
        Utn57WrittenUnit::MVS => WrittenUnit::Mvs,
    }
}

/// Map a UTN #57 joining position onto its `mongol-norm` counterpart.
const fn unit_position(position: Utn57Position) -> UnitPosition {
    match position {
        Utn57Position::Isol => UnitPosition::Isol,
        Utn57Position::Init => UnitPosition::Init,
        Utn57Position::Medi => UnitPosition::Medi,
        Utn57Position::Fina => UnitPosition::Fina,
        Utn57Position::Control => UnitPosition::Control,
    }
}

/// Map one positioned record onto its `mongol-norm` counterpart.
const fn positioned_written_unit(unit: Utn57PositionedWrittenUnit) -> PositionedWrittenUnit {
    PositionedWrittenUnit::new(
        written_unit(unit.written_unit),
        unit_position(unit.position),
    )
}

/// Encode one run of positioned UTN #57 written units as canonical Mongolian Unicode.
///
/// Records are handed to `mongol-norm`'s
/// [`Shaper::normalize_positioned_written_units`], which supplies the implicit
/// ZWJ joining context an incomplete chain needs and returns the FVS-pinned
/// canonical encoding.
///
/// # Errors
///
/// Returns the [`mongol_norm::Error`] the backend reports — for example
/// [`Error::UnsupportedPositionedUnit`] when a `(unit, position)` pair is
/// outside the authoritative HUD inventory, or [`Error::ChainPositionMismatch`]
/// when declared positions disagree with the chain they form. Its record
/// indices refer to the expanded written-unit sequence, not to `units`.
pub fn normalize_positioned_written_units(
    units: &[Utn57PositionedWrittenUnit],
) -> Result<String, Error> {
    let records: Vec<PositionedWrittenUnit> =
        units.iter().copied().map(positioned_written_unit).collect();
    shaper().normalize_positioned_written_units(&records)
}

/// Which reviewed UTN #57 unit is this `mongol-norm` unit?
///
/// The inverse of [`written_unit`], resolved by searching the reviewed
/// inventory rather than by a second `match`, so the two cannot drift apart.
/// Returns `None` for a unit outside the inventory, such as `Zwj`.
fn utn57_written_unit(unit: WrittenUnit) -> Option<Utn57WrittenUnit> {
    UTN57_POSITIONED_WRITTEN_UNITS
        .iter()
        .map(|record| record.written_unit)
        .find(|candidate| written_unit(*candidate) == unit)
}

/// Is this reviewed `(unit, position)` pair in the HUD inventory?
fn is_reviewed(unit: Utn57WrittenUnit, position: Utn57Position) -> bool {
    UTN57_POSITIONED_WRITTEN_UNITS.contains(&Utn57PositionedWrittenUnit::new(unit, position))
}

/// The joining position of the `start`-th of `count` slots.
///
/// Mirrors `mongol-norm`'s own `slot_position` for a one-slot unit, so a
/// sequence shaped here re-encodes to the shape it came from.
const fn slot_position(start: usize, count: usize) -> Utn57Position {
    match (start, count) {
        (0, 1) => Utn57Position::Isol,
        (0, _) => Utn57Position::Init,
        _ if start + 1 == count => Utn57Position::Fina,
        _ => Utn57Position::Medi,
    }
}

/// Failure while shaping Mongolian text into positioned UTN #57 units.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Utn57ShapeError {
    /// `mongol-norm` could not shape the text.
    Shape(Error),
    /// A shaped unit is outside the reviewed UTN #57 inventory.
    UnsupportedWrittenUnit {
        /// Position of the unit in the shaped sequence.
        index: usize,
        /// The unit with no reviewed UTN #57 counterpart.
        unit: WrittenUnit,
    },
}

impl std::fmt::Display for Utn57ShapeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Shape(error) => error.fmt(formatter),
            Self::UnsupportedWrittenUnit { index, unit } => write!(
                formatter,
                "written unit {unit} at index {index} is outside the reviewed UTN #57 inventory"
            ),
        }
    }
}

impl std::error::Error for Utn57ShapeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Shape(error) => Some(error),
            Self::UnsupportedWrittenUnit { .. } => None,
        }
    }
}

impl From<Error> for Utn57ShapeError {
    fn from(error: Error) -> Self {
        Self::Shape(error)
    }
}

/// Shape one Mongolian word into positioned UTN #57 written units.
///
/// `mongol-norm` returns a flat written-unit sequence; this recovers the
/// joining position of each unit the way the positioned encoder infers it.
/// The sequence is split at the structural units, and each remaining chain is
/// padded by one slot on any side a joiner (`Nirugu` or `Zwj`) sits against, so
/// a chain that continues past its edge is positioned as if it did. `Mvs` and
/// `Nirugu` become `Control` records; `Zwj` supplies joining context and is not
/// itself a record.
///
/// A position of `Isol` that the reviewed inventory does not carry falls back to
/// `Init`, matching how isolated forms borrow the initial glyph.
///
/// # Errors
///
/// Returns [`Utn57ShapeError::Shape`] when `mongol-norm` rejects the text — it
/// accepts only Mongolian letters, FVS, MVS, NNBSP, nirugu and ZWJ — and
/// [`Utn57ShapeError::UnsupportedWrittenUnit`] for a shaped unit outside the
/// reviewed UTN #57 inventory.
pub fn shape_utn57_positioned_written_units(
    text: &str,
) -> Result<Vec<Utn57PositionedWrittenUnit>, Utn57ShapeError> {
    let shape = shaper().shape(text)?;
    let is_joiner = |unit: WrittenUnit| matches!(unit, WrittenUnit::Nirugu | WrittenUnit::Zwj);

    let mut records = Vec::with_capacity(shape.len());
    let mut index = 0;
    while index < shape.len() {
        let unit = shape[index];
        if matches!(
            unit,
            WrittenUnit::Mvs | WrittenUnit::Nirugu | WrittenUnit::Zwj
        ) {
            if unit != WrittenUnit::Zwj {
                let written = utn57_written_unit(unit)
                    .ok_or(Utn57ShapeError::UnsupportedWrittenUnit { index, unit })?;
                records.push(Utn57PositionedWrittenUnit::new(
                    written,
                    Utn57Position::Control,
                ));
            }
            index += 1;
            continue;
        }
        // A chain runs to the next structural unit.
        let start = index;
        while index < shape.len()
            && !matches!(
                shape[index],
                WrittenUnit::Mvs | WrittenUnit::Nirugu | WrittenUnit::Zwj
            )
        {
            index += 1;
        }
        let chain = &shape[start..index];
        let joined_left = start > 0 && is_joiner(shape[start - 1]);
        let joined_right = index < shape.len() && is_joiner(shape[index]);
        let count = chain.len() + usize::from(joined_left) + usize::from(joined_right);
        for (offset, unit) in chain.iter().enumerate() {
            let written =
                utn57_written_unit(*unit).ok_or(Utn57ShapeError::UnsupportedWrittenUnit {
                    index: start + offset,
                    unit: *unit,
                })?;
            let mut position = slot_position(offset + usize::from(joined_left), count);
            if position == Utn57Position::Isol && !is_reviewed(written, position) {
                position = Utn57Position::Init;
            }
            records.push(Utn57PositionedWrittenUnit::new(written, position));
        }
    }
    Ok(records)
}

/// Failure while converting one ZVVNMOD run to canonical Mongolian Unicode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Utn57RunConversionError {
    /// ZVVNMOD → positioned UTN #57 unit conversion failed.
    Conversion(Utn57ConversionError),
    /// `mongol-norm` could not encode the positioned units.
    Normalize(Error),
}

impl std::fmt::Display for Utn57RunConversionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Conversion(error) => error.fmt(formatter),
            Self::Normalize(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for Utn57RunConversionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Conversion(error) => Some(error),
            Self::Normalize(error) => Some(error),
        }
    }
}

impl From<Utn57ConversionError> for Utn57RunConversionError {
    fn from(error: Utn57ConversionError) -> Self {
        Self::Conversion(error)
    }
}

impl From<Error> for Utn57RunConversionError {
    fn from(error: Error) -> Self {
        Self::Normalize(error)
    }
}

/// Convert one uninterrupted ZVVNMOD PUA run to canonical Mongolian Unicode.
///
/// Every character is taken as a ZVVNMOD code. Use
/// [`convert_zvvnmod_to_utn57`](crate::convert_zvvnmod_to_utn57) for complete
/// text, which splits ZVVNMOD runs from passthrough characters first.
///
/// # Errors
///
/// Returns [`Utn57RunConversionError`] for either conversion stage.
pub fn convert_zvvnmod_run_to_utn57_text(input: &str) -> Result<String, Utn57RunConversionError> {
    let codes: Vec<_> = input
        .chars()
        .map(|character| ZvvnmodCode(character as u32))
        .collect();
    let units = convert_zvvnmod_run(&codes)?;
    Ok(normalize_positioned_written_units(&units)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn singleton_o_init_takes_a_trailing_zwj() {
        let units = [Utn57PositionedWrittenUnit::new(
            Utn57WrittenUnit::O,
            Utn57Position::Init,
        )];

        assert_eq!(
            normalize_positioned_written_units(&units).unwrap(),
            "\u{1824}\u{180b}\u{200d}"
        );
    }

    #[test]
    fn an_empty_request_normalizes_to_empty_text() {
        assert_eq!(normalize_positioned_written_units(&[]).unwrap(), "");
    }

    #[test]
    fn a_control_unit_outside_control_position_is_rejected() {
        let units = [Utn57PositionedWrittenUnit::new(
            Utn57WrittenUnit::MVS,
            Utn57Position::Medi,
        )];

        assert_eq!(
            normalize_positioned_written_units(&units).unwrap_err(),
            Error::ControlRequiresControlPosition {
                index: 0,
                unit: WrittenUnit::Mvs,
            }
        );
    }

    #[test]
    fn declared_positions_must_match_the_chain_they_form() {
        let units = [
            Utn57PositionedWrittenUnit::new(Utn57WrittenUnit::B, Utn57Position::Init),
            Utn57PositionedWrittenUnit::new(Utn57WrittenUnit::A, Utn57Position::Init),
        ];

        assert_eq!(
            normalize_positioned_written_units(&units).unwrap_err(),
            Error::ChainPositionMismatch
        );
    }

    #[test]
    fn the_shaper_is_built_once_for_the_whole_process() {
        assert!(std::ptr::eq(shaper(), shaper()));
    }
}

#[cfg(test)]
mod shaping_tests {
    use super::*;

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

    #[test]
    fn a_word_positions_every_unit_by_its_place_in_the_chain() {
        // ᠮᠣᠩᠭᠣᠯ shapes to M+O+A+G+Hx+O+L.
        assert_eq!(
            spell(
                &shape_utn57_positioned_written_units(
                    "\u{182E}\u{1823}\u{1829}\u{182D}\u{1823}\u{182F}"
                )
                .unwrap()
            ),
            "M:init O:medi A:medi G:medi Hx:medi O:medi L:fina"
        );
    }

    #[test]
    fn mvs_becomes_a_control_record_and_splits_the_chain() {
        // ᠰᠢᠨ᠎ᠡ shapes to S+I+N+Mvs+Aa.
        assert_eq!(
            spell(
                &shape_utn57_positioned_written_units("\u{1830}\u{1822}\u{1828}\u{180E}\u{1821}")
                    .unwrap()
            ),
            "S:init I:medi N:fina Mvs Aa:isol"
        );
    }

    #[test]
    fn a_trailing_zwj_makes_a_single_unit_initial_rather_than_isolated() {
        // ᠤ+FVS1+ZWJ shapes to O+Zwj: the joiner pads the chain's right edge.
        assert_eq!(
            spell(&shape_utn57_positioned_written_units("\u{1824}\u{180B}\u{200D}").unwrap()),
            "O:init"
        );
    }

    #[test]
    fn a_lone_letter_is_isolated() {
        assert_eq!(
            spell(&shape_utn57_positioned_written_units("\u{1822}").unwrap()),
            "A:init I:fina"
        );
    }

    /// The property the inversion owes: what it recovers must re-encode to the
    /// text it was shaped from.
    #[test]
    fn shaping_and_normalizing_are_inverses() {
        let words = [
            "\u{182E}\u{1823}\u{1829}\u{182D}\u{1823}\u{182F}", // ᠮᠣᠩᠭᠣᠯ
            "\u{182A}\u{1822}\u{1834}\u{1822}\u{182D}",         // ᠪᠢᠴᠢᠭ
            "\u{1824}\u{182F}\u{1824}\u{1830}",                 // ᠤᠯᠤᠰ
            "\u{1830}\u{1822}\u{1828}\u{180E}\u{1821}",         // ᠰᠢᠨ᠎ᠡ
            "\u{182A}\u{1820}\u{182D}\u{180E}\u{1820}",         // ᠪᠠᠭ᠎ᠠ
            "\u{182C}\u{1821}\u{182F}\u{1821}",                 // ᠬᠡᠯᠡ
            "\u{1828}\u{1824}\u{182D}\u{1824}\u{1833}",         // ᠨᠤᠭᠤᠳ
        ];
        for word in words {
            let records = shape_utn57_positioned_written_units(word).unwrap();
            let encoded = normalize_positioned_written_units(&records).unwrap();
            assert_eq!(
                shaper().shape(&encoded).unwrap(),
                shaper().shape(word).unwrap(),
                "{word} reshaped differently through {}",
                spell(&records)
            );
        }
    }

    #[test]
    fn a_unit_outside_the_reviewed_inventory_reports_its_index() {
        // Todo's E has no UTN #57 counterpart, and MNG shaping rejects it, so
        // the error path here is the shaping one.
        assert!(matches!(
            shape_utn57_positioned_written_units("x"),
            Err(Utn57ShapeError::Shape(_))
        ));
    }
}
