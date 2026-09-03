use crate::normalize::{normalize_positioned_written_units, shape_utn57_positioned_written_units};
use crate::{
    classify_zvvnmod_text_character, convert_utn57_run_to_zvvnmod, convert_zvvnmod_run,
    zvvnmod_code, Utn57ConversionError, Utn57PositionedWrittenUnit, Utn57ReverseError,
    Utn57ShapeError, ZvvnmodCode, ZvvnmodTextCharacterKind,
};
use mongol_norm::is_mongolian_word_char;
use std::error::Error;
use std::fmt;

/// Failure while converting complete ZVVNMOD text to canonical UTN #57 output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Utn57TextConversionError {
    /// ZVVNMOD → positioned UTN #57 unit conversion failed.
    Conversion(Utn57ConversionError),
    /// `mongol-norm` could not encode the positioned units of a run.
    Normalize(mongol_norm::Error),
}

impl fmt::Display for Utn57TextConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Conversion(error) => error.fmt(formatter),
            Self::Normalize(error) => error.fmt(formatter),
        }
    }
}

impl Error for Utn57TextConversionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Conversion(error) => Some(error),
            Self::Normalize(error) => Some(error),
        }
    }
}

impl From<Utn57ConversionError> for Utn57TextConversionError {
    fn from(error: Utn57ConversionError) -> Self {
        Self::Conversion(error)
    }
}

impl From<mongol_norm::Error> for Utn57TextConversionError {
    fn from(error: mongol_norm::Error) -> Self {
        Self::Normalize(error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ClassifiedTextPart {
    ZvvnmodRun(Vec<ZvvnmodCode>),
    Passthrough(String),
}

fn append_zvvnmod_code(parts: &mut Vec<ClassifiedTextPart>, code: ZvvnmodCode) {
    if let Some(ClassifiedTextPart::ZvvnmodRun(run)) = parts.last_mut() {
        run.push(code);
    } else {
        parts.push(ClassifiedTextPart::ZvvnmodRun(vec![code]));
    }
}

fn append_passthrough(parts: &mut Vec<ClassifiedTextPart>, character: char) {
    if let Some(ClassifiedTextPart::Passthrough(text)) = parts.last_mut() {
        text.push(character);
    } else {
        parts.push(ClassifiedTextPart::Passthrough(character.to_string()));
    }
}

fn classify_complete_text(input: &str) -> Vec<ClassifiedTextPart> {
    let mut parts = Vec::new();
    for character in input.chars() {
        match classify_zvvnmod_text_character(character) {
            ZvvnmodTextCharacterKind::Shape => {
                append_zvvnmod_code(
                    &mut parts,
                    zvvnmod_code(character).expect("shape classification has a code"),
                );
            }
            ZvvnmodTextCharacterKind::LegacyControl => {
                // Legacy PUA FVS1-FVS4/MVS controls are excluded without
                // breaking the surrounding ZVVNMOD run.
            }
            ZvvnmodTextCharacterKind::Passthrough => {
                append_passthrough(&mut parts, character);
            }
        }
    }
    parts
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ReconstructionStep {
    NormalizedRun(usize),
    Passthrough(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NormalizationPlan {
    positioned_written_unit_runs: Vec<Vec<Utn57PositionedWrittenUnit>>,
    reconstruction: Vec<ReconstructionStep>,
}

fn build_normalization_plan(input: &str) -> Result<NormalizationPlan, Utn57TextConversionError> {
    let mut positioned_written_unit_runs = Vec::new();
    let mut reconstruction = Vec::new();
    for part in classify_complete_text(input) {
        match part {
            ClassifiedTextPart::ZvvnmodRun(run) => {
                let positioned = convert_zvvnmod_run(&run)?;
                let run_index = positioned_written_unit_runs.len();
                positioned_written_unit_runs.push(positioned);
                reconstruction.push(ReconstructionStep::NormalizedRun(run_index));
            }
            ClassifiedTextPart::Passthrough(text) => {
                reconstruction.push(ReconstructionStep::Passthrough(text));
            }
        }
    }
    Ok(NormalizationPlan {
        positioned_written_unit_runs,
        reconstruction,
    })
}

fn reconstruct_complete_text(
    reconstruction: Vec<ReconstructionStep>,
    normalized_runs: Vec<String>,
) -> String {
    let mut output = String::new();
    for step in reconstruction {
        match step {
            ReconstructionStep::NormalizedRun(index) => output.push_str(&normalized_runs[index]),
            ReconstructionStep::Passthrough(text) => output.push_str(&text),
        }
    }
    output
}

/// Convert complete text containing ZVVNMOD shape runs to canonical UTN #57 output.
///
/// Formal ZVVNMOD shape runs are normalized in process by the `mongol-norm`
/// crate. Characters outside the formal ZVVNMOD shape inventory,
/// including punctuation, digits, whitespace, ordinary Unicode, emoji, and
/// non-ZVVNMOD private-use values, preserve their order and code points.
/// Legacy ZVVNMOD `U+E140..=U+E144` FVS1-FVS4/MVS controls are excluded.
///
/// # Errors
///
/// Returns [`Utn57TextConversionError`] for either conversion stage.
pub fn convert_zvvnmod_to_utn57(input: &str) -> Result<String, Utn57TextConversionError> {
    let plan = build_normalization_plan(input)?;
    let mut normalized_runs = Vec::with_capacity(plan.positioned_written_unit_runs.len());
    for run in &plan.positioned_written_unit_runs {
        normalized_runs.push(normalize_positioned_written_units(run)?);
    }
    Ok(reconstruct_complete_text(
        plan.reconstruction,
        normalized_runs,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_outside_the_zvvnmod_shape_inventory_is_preserved() {
        let input = "English 中 😀\t\r\n\u{1802}\u{1810}\u{E23F}";
        assert_eq!(convert_zvvnmod_to_utn57(input).unwrap(), input);
    }

    #[test]
    fn plain_text_zwj_is_preserved_as_passthrough() {
        assert_eq!(
            convert_zvvnmod_to_utn57("a\u{200D}b").unwrap(),
            "a\u{200D}b"
        );
    }

    #[test]
    fn non_zvvnmod_private_use_passes_through_unchanged() {
        let input = "a\u{E145}\u{F0000}\u{100000}b";
        assert_eq!(convert_zvvnmod_to_utn57(input).unwrap(), input);
    }

    #[test]
    fn standard_controls_pass_through_and_delimit_zvvnmod_runs() {
        let controls = "\u{180A}\u{180E}\u{202F}";
        let plan = build_normalization_plan(&format!("\u{E001}{controls}\u{E00D}")).unwrap();

        assert_eq!(plan.positioned_written_unit_runs.len(), 2);
        assert_eq!(
            plan.reconstruction,
            vec![
                ReconstructionStep::NormalizedRun(0),
                ReconstructionStep::Passthrough(controls.to_owned()),
                ReconstructionStep::NormalizedRun(1),
            ]
        );
    }

    #[test]
    fn normalization_plan_keeps_passthrough_outside_positioned_runs() {
        let plan = build_normalization_plan("\u{E001}\u{200D}\u{E00D}").unwrap();

        assert_eq!(plan.positioned_written_unit_runs.len(), 2);
        assert_eq!(
            plan.reconstruction,
            vec![
                ReconstructionStep::NormalizedRun(0),
                ReconstructionStep::Passthrough("\u{200D}".to_owned()),
                ReconstructionStep::NormalizedRun(1),
            ]
        );
        assert_eq!(
            reconstruct_complete_text(
                plan.reconstruction,
                vec!["left\u{200D}".to_owned(), "right".to_owned()],
            ),
            "left\u{200D}\u{200D}right"
        );
    }
}

/// Failure while converting complete Mongolian text to ZVVNMOD output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ZvvnmodTextConversionError {
    /// A Mongolian word could not be shaped into positioned UTN #57 units.
    Shape(Utn57ShapeError),
    /// Positioned UTN #57 units could not be spelled in ZVVNMOD.
    Reverse(Utn57ReverseError),
}

impl fmt::Display for ZvvnmodTextConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Shape(error) => error.fmt(formatter),
            Self::Reverse(error) => error.fmt(formatter),
        }
    }
}

impl Error for ZvvnmodTextConversionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Shape(error) => Some(error),
            Self::Reverse(error) => Some(error),
        }
    }
}

impl From<Utn57ShapeError> for ZvvnmodTextConversionError {
    fn from(error: Utn57ShapeError) -> Self {
        Self::Shape(error)
    }
}

impl From<Utn57ReverseError> for ZvvnmodTextConversionError {
    fn from(error: Utn57ReverseError) -> Self {
        Self::Reverse(error)
    }
}

/// Convert complete text containing Mongolian words to ZVVNMOD output.
///
/// Runs of Mongolian word characters — letters, FVS, MVS, NNBSP, nirugu and
/// ZWJ — are shaped, positioned, and spelled in ZVVNMOD. Every other character
/// preserves its order and code point, mirroring
/// [`convert_zvvnmod_to_utn57`]'s treatment of text outside the ZVVNMOD shape
/// inventory.
///
/// # Errors
///
/// Returns [`ZvvnmodTextConversionError`] for either conversion stage. A unit
/// the ZVVNMOD font has no glyph for is reported rather than replaced by a near
/// glyph, so the output never silently differs from the input.
pub fn convert_utn57_to_zvvnmod(input: &str) -> Result<String, ZvvnmodTextConversionError> {
    fn flush(word: &mut String, output: &mut String) -> Result<(), ZvvnmodTextConversionError> {
        if word.is_empty() {
            return Ok(());
        }
        let records = shape_utn57_positioned_written_units(word)?;
        for code in convert_utn57_run_to_zvvnmod(&records)? {
            output.push(code.as_char().expect("ZVVNMOD codes are scalar values"));
        }
        word.clear();
        Ok(())
    }

    let mut output = String::with_capacity(input.len());
    let mut word = String::new();
    for character in input.chars() {
        if is_mongolian_word_char(character) {
            word.push(character);
        } else {
            flush(&mut word, &mut output)?;
            output.push(character);
        }
    }
    flush(&mut word, &mut output)?;
    Ok(output)
}
