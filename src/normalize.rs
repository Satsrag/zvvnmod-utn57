use crate::{
    convert_zvvnmod_run, Utn57ConversionError, Utn57Position, Utn57PositionedWrittenUnit,
    Utn57WrittenUnit, ZvvnmodCode,
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
