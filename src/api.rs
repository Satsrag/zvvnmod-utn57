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

/// UTN #57 `MVS`, the written-unit spelling of a detached-suffix boundary.
///
/// The inverse of [`ZVVNMOD_SUFFIX_SEPARATOR`](crate::ZVVNMOD_SUFFIX_SEPARATOR):
/// what the reverse direction writes as `U+202F` is read back as this.
const UTN57_SUFFIX_SEPARATOR: char = '\u{180E}';

#[derive(Clone, Debug, PartialEq, Eq)]
enum ClassifiedTextPart {
    ZvvnmodRun(Vec<ZvvnmodCode>),
    /// A detached-suffix boundary, which delimits the runs on either side of it.
    ///
    /// `separators` is how many `U+202F` the input wrote there. Two or more in a
    /// row still mark one boundary — there is nothing between them to separate —
    /// so they fold into one part and are reported, not multiplied.
    SuffixSeparator {
        separators: usize,
    },
    Passthrough(String),
}

fn append_suffix_separator(parts: &mut Vec<ClassifiedTextPart>) {
    if let Some(ClassifiedTextPart::SuffixSeparator { separators }) = parts.last_mut() {
        *separators += 1;
    } else {
        parts.push(ClassifiedTextPart::SuffixSeparator { separators: 1 });
    }
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
            ZvvnmodTextCharacterKind::SuffixSeparator => {
                append_suffix_separator(&mut parts);
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
    /// One boundary, however many separators the input wrote there.
    SuffixSeparator {
        separators: usize,
    },
    Passthrough(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NormalizationPlan {
    /// The classified codes of every ZVVNMOD run, in input order — what a
    /// warning names the run by.
    zvvnmod_runs: Vec<Vec<ZvvnmodCode>>,
    positioned_written_unit_runs: Vec<Vec<Utn57PositionedWrittenUnit>>,
    reconstruction: Vec<ReconstructionStep>,
}

fn build_normalization_plan(input: &str) -> Result<NormalizationPlan, Utn57TextConversionError> {
    let mut zvvnmod_runs = Vec::new();
    let mut positioned_written_unit_runs = Vec::new();
    let mut reconstruction = Vec::new();
    for part in classify_complete_text(input) {
        match part {
            ClassifiedTextPart::ZvvnmodRun(run) => {
                let positioned = convert_zvvnmod_run(&run)?;
                let run_index = positioned_written_unit_runs.len();
                zvvnmod_runs.push(run);
                positioned_written_unit_runs.push(positioned);
                reconstruction.push(ReconstructionStep::NormalizedRun(run_index));
            }
            ClassifiedTextPart::SuffixSeparator { separators } => {
                reconstruction.push(ReconstructionStep::SuffixSeparator { separators });
            }
            ClassifiedTextPart::Passthrough(text) => {
                reconstruction.push(ReconstructionStep::Passthrough(text));
            }
        }
    }
    Ok(NormalizationPlan {
        zvvnmod_runs,
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
            ReconstructionStep::SuffixSeparator { .. } => output.push(UTN57_SUFFIX_SEPARATOR),
            ReconstructionStep::Passthrough(text) => output.push_str(&text),
        }
    }
    output
}

/// Something the conversion could do only by going beyond what the input said.
///
/// A warning never changes the result: the text is still the ink the ZVVNMOD
/// input describes. It tells the caller *how* that ink had to be spelled, which
/// is what surfaces a gap in the hub inventory.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Utn57ConversionWarning {
    /// A run was spelled with a ZWJ the input did not carry.
    ///
    /// Input ZWJ is passthrough and never enters a run, so every joiner inside
    /// a normalized run is one `mongol-norm` invented: the run begins or ends
    /// with a joined-form glyph (a `medi` at the start, a `medi` or `init` at
    /// the end) and ZVVNMOD has no unjoined form of it. `U+E09C` (`G m O f`)
    /// standing for a whole word was the first such gap (Satsrag/meco-rust#32);
    /// it is now spelled by `U+E096`. A lone joined-form glyph a source really
    /// did write is reported the same way — the ink is right either way.
    InventedZwj {
        /// Ordinal of the run among the input's ZVVNMOD runs, from zero.
        run: usize,
        /// The run's codes as classified from the input, legacy controls dropped.
        codes: Vec<ZvvnmodCode>,
        /// How many `U+200D` the run's spelling carries.
        count: usize,
    },
    /// A detached-suffix boundary written with more than one separator.
    ///
    /// ZVVNMOD spells the boundary between a stem and its detached suffix as
    /// `U+202F`; UTN #57 as one `MVS`, which can stand only once between the
    /// stem's final letter and the suffix's first. Two or more separators in a
    /// row still mark one boundary — there is nothing between them to separate
    /// — so one MVS is written and the repetition is reported here, in case it
    /// was not a slip (Satsrag/meco-rust#40).
    CollapsedSuffixSeparators {
        /// Ordinal of the boundary among the input's boundaries, from zero.
        boundary: usize,
        /// How many separators the input wrote there.
        separators: usize,
    },
}

impl fmt::Display for Utn57ConversionWarning {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InventedZwj { run, codes, count } => {
                write!(formatter, "ZVVNMOD run {run} (")?;
                for (index, code) in codes.iter().enumerate() {
                    if index > 0 {
                        formatter.write_str(" ")?;
                    }
                    write!(formatter, "U+{:04X}", code.codepoint())?;
                }
                write!(
                    formatter,
                    ") is spelled with {count} invented ZWJ: it begins or ends with a \
                     joined-form glyph that ZVVNMOD has no unjoined form of"
                )
            }
            Self::CollapsedSuffixSeparators {
                boundary,
                separators,
            } => write!(
                formatter,
                "suffix boundary {boundary} is written with {separators} separators; one MVS \
                 is emitted, since a boundary can only be crossed once"
            ),
        }
    }
}

/// The result of [`convert_zvvnmod_to_utn57_with_warnings`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Utn57Conversion {
    /// The canonical UTN #57 text — exactly what [`convert_zvvnmod_to_utn57`] returns.
    pub text: String,
    /// What the conversion had to do beyond what the input said, in input order.
    pub warnings: Vec<Utn57ConversionWarning>,
}

/// Convert complete text containing ZVVNMOD shape runs to canonical UTN #57
/// output, reporting every run whose spelling needed an invented ZWJ.
///
/// The text is the one [`convert_zvvnmod_to_utn57`] returns; see there for
/// what is converted and what passes through. The warnings are
/// [`Utn57ConversionWarning`]s, one per affected run, in input order.
///
/// # Errors
///
/// Returns [`Utn57TextConversionError`] for either conversion stage.
pub fn convert_zvvnmod_to_utn57_with_warnings(
    input: &str,
) -> Result<Utn57Conversion, Utn57TextConversionError> {
    let plan = build_normalization_plan(input)?;
    let mut normalized_runs = vec![String::new(); plan.positioned_written_unit_runs.len()];
    let mut warnings = Vec::new();
    let mut boundary = 0;
    // Walk the steps rather than the runs, so the warnings come out in input order.
    for step in &plan.reconstruction {
        match step {
            ReconstructionStep::NormalizedRun(run) => {
                let normalized =
                    normalize_positioned_written_units(&plan.positioned_written_unit_runs[*run])?;
                let count = normalized.matches(INVENTED_JOINER).count();
                if count > 0 {
                    warnings.push(Utn57ConversionWarning::InventedZwj {
                        run: *run,
                        codes: plan.zvvnmod_runs[*run].clone(),
                        count,
                    });
                }
                normalized_runs[*run] = normalized;
            }
            ReconstructionStep::SuffixSeparator { separators } => {
                if *separators > 1 {
                    warnings.push(Utn57ConversionWarning::CollapsedSuffixSeparators {
                        boundary,
                        separators: *separators,
                    });
                }
                boundary += 1;
            }
            ReconstructionStep::Passthrough(_) => {}
        }
    }
    Ok(Utn57Conversion {
        text: reconstruct_complete_text(plan.reconstruction, normalized_runs),
        warnings,
    })
}

/// `U+200D`: input ZWJ never enters a run, so one inside a normalized run is invented.
const INVENTED_JOINER: char = '\u{200D}';

/// Convert complete text containing ZVVNMOD shape runs to canonical UTN #57 output.
///
/// Formal ZVVNMOD shape runs are normalized in process by the `mongol-norm`
/// crate. `U+202F`, the detached-suffix boundary ZVVNMOD writes between a stem
/// and its detached suffix, delimits the runs on either side of it and is read
/// back as UTN #57 `MVS` — one MVS however many `U+202F` were written in a row,
/// since a boundary can only be crossed once. Characters outside the formal ZVVNMOD shape inventory,
/// including punctuation, digits, whitespace, ordinary Unicode, emoji, and
/// non-ZVVNMOD private-use values, preserve their order and code points — the
/// `U+0020` ZVVNMOD writes between words included.
/// Legacy ZVVNMOD `U+E140..=U+E144` FVS1-FVS4/MVS controls are excluded.
///
/// A run the normalizer could spell only by inventing a ZWJ is converted all
/// the same; [`convert_zvvnmod_to_utn57_with_warnings`] reports those runs.
///
/// # Errors
///
/// Returns [`Utn57TextConversionError`] for either conversion stage.
pub fn convert_zvvnmod_to_utn57(input: &str) -> Result<String, Utn57TextConversionError> {
    convert_zvvnmod_to_utn57_with_warnings(input).map(|conversion| conversion.text)
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
        let controls = "\u{180A}\u{180E}";
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
    fn repeated_suffix_separators_fold_into_one_step() {
        let plan = build_normalization_plan("\u{E001}\u{202F}\u{202F}\u{202F}\u{E00D}").unwrap();

        assert_eq!(plan.positioned_written_unit_runs.len(), 2);
        assert_eq!(
            plan.reconstruction,
            vec![
                ReconstructionStep::NormalizedRun(0),
                ReconstructionStep::SuffixSeparator { separators: 3 },
                ReconstructionStep::NormalizedRun(1),
            ]
        );
    }

    #[test]
    fn the_suffix_separator_delimits_runs_as_its_own_step() {
        let plan = build_normalization_plan("\u{E001}\u{202F}\u{E00D}").unwrap();

        assert_eq!(plan.positioned_written_unit_runs.len(), 2);
        assert_eq!(
            plan.reconstruction,
            vec![
                ReconstructionStep::NormalizedRun(0),
                ReconstructionStep::SuffixSeparator { separators: 1 },
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
/// ZWJ — are shaped, positioned, and spelled in ZVVNMOD. A detached-suffix
/// boundary the chachlag rules did not consume is spelled `U+202F`, so
/// [`convert_zvvnmod_to_utn57`] can tell it from the `U+0020` between words.
/// Every other character preserves its order and code point, mirroring
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
