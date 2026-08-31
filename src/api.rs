use crate::command_bridge::{
    normalize_positioned_written_unit_runs_with_mongol_norm, MongolNormCommandError,
};
use crate::{
    classify_zvvnmod_text_character, convert_zvvnmod_run, zvvnmod_code, Utn57PositionedWrittenUnit,
    ZvvnmodCode, ZvvnmodTextCharacterKind,
};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
enum Utn57TextConversionErrorKind {
    Backend(MongolNormCommandError),
}

/// Failure while converting complete ZVVNMOD text to canonical UTN #57 output.
///
/// The concrete normalization backend is intentionally kept behind this
/// backend-neutral error boundary.
#[derive(Debug)]
pub struct Utn57TextConversionError {
    kind: Utn57TextConversionErrorKind,
}

impl fmt::Display for Utn57TextConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            Utn57TextConversionErrorKind::Backend(error) => error.fmt(formatter),
        }
    }
}

impl Error for Utn57TextConversionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.kind {
            Utn57TextConversionErrorKind::Backend(error) => Some(error),
        }
    }
}

impl From<MongolNormCommandError> for Utn57TextConversionError {
    fn from(source: MongolNormCommandError) -> Self {
        Self {
            kind: Utn57TextConversionErrorKind::Backend(source),
        }
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

fn convert_classified_zvvnmod_run(
    run: Vec<ZvvnmodCode>,
) -> Result<Vec<Utn57PositionedWrittenUnit>, Utn57TextConversionError> {
    convert_zvvnmod_run(&run)
        .map_err(MongolNormCommandError::from)
        .map_err(Utn57TextConversionError::from)
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
                let positioned = convert_classified_zvvnmod_run(run)?;
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
/// Formal ZVVNMOD shape runs are converted through one external normalization
/// process per call. Characters outside the formal ZVVNMOD shape inventory,
/// including punctuation, digits, whitespace, ordinary Unicode, emoji, and
/// non-ZVVNMOD private-use values, preserve their order and code points.
/// Legacy ZVVNMOD `U+E140..=U+E144` FVS1-FVS4/MVS controls are excluded.
pub fn convert_zvvnmod_to_utn57(input: &str) -> Result<String, Utn57TextConversionError> {
    let plan = build_normalization_plan(input)?;
    let normalized_runs = if plan.positioned_written_unit_runs.is_empty() {
        Vec::new()
    } else {
        normalize_positioned_written_unit_runs_with_mongol_norm(&plan.positioned_written_unit_runs)?
    };
    Ok(reconstruct_complete_text(
        plan.reconstruction,
        normalized_runs,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_outside_the_zvvnmod_shape_inventory_needs_no_backend() {
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
    fn non_zvvnmod_private_use_passes_through_without_backend_startup() {
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
