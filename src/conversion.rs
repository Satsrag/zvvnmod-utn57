use crate::{
    discard_legacy_controls, replace_ir_fina, IrFinaReplacementError, Utn57WrittenUnit,
    ZvvnmodCode, ZVVNMOD_CODE_DECOMPOSITIONS, ZVVNMOD_TO_UTN57_MAPPINGS,
};

/// A ZVVNMOD code that has no reviewed UTN #57 mapping at an input index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Utn57MappingError {
    /// Index in the input run after preprocessing and decomposition.
    pub index: usize,
    /// Unmapped ZVVNMOD code.
    pub code: ZvvnmodCode,
}

impl std::fmt::Display for Utn57MappingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "unmapped ZVVNMOD code U+{:04X} at index {}",
            self.code.codepoint(),
            self.index
        )
    }
}

impl std::error::Error for Utn57MappingError {}

/// Failure in an ordered ZVVNMOD → UTN #57 conversion stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Utn57ConversionError {
    /// `Ir_fina` could not replace its preceding form.
    IrFina(IrFinaReplacementError),
    /// A preprocessed/decomposed code had no reviewed mapping.
    Mapping(Utn57MappingError),
}

impl std::fmt::Display for Utn57ConversionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IrFina(error) => error.fmt(formatter),
            Self::Mapping(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for Utn57ConversionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IrFina(error) => Some(error),
            Self::Mapping(error) => Some(error),
        }
    }
}

impl From<IrFinaReplacementError> for Utn57ConversionError {
    fn from(error: IrFinaReplacementError) -> Self {
        Self::IrFina(error)
    }
}

impl From<Utn57MappingError> for Utn57ConversionError {
    fn from(error: Utn57MappingError) -> Self {
        Self::Mapping(error)
    }
}

/// Resolution for the UTN #57 K/K2 units that share ZVVNMOD shapes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Utn57KVariant {
    /// Emit K, the canonical default.
    #[default]
    K,
    /// Emit K2 when the caller has nominal/context information requiring it.
    K2,
}

/// Options for reviewed ZVVNMOD → UTN #57 mapping replacement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Utn57ConversionOptions {
    /// Resolve the shared K/K2 ZVVNMOD shapes.
    pub k_variant: Utn57KVariant,
}

/// Convert one ZVVNMOD written-form run to reviewed UTN #57 units.
///
/// The function discards legacy controls, applies `Ir_fina` replacement,
/// decomposes general merged codes, and then applies reviewed longest-match
/// mapping. It does not yet reconstruct MVS/ZWJ or particle boundaries.
pub fn convert_zvvnmod_run(
    input: &[ZvvnmodCode],
) -> Result<Vec<Utn57WrittenUnit>, Utn57ConversionError> {
    convert_zvvnmod_run_with_options(input, Utn57ConversionOptions::default())
}

/// Convert a run while applying explicit K/K2 ambiguity options.
pub fn convert_zvvnmod_run_with_options(
    input: &[ZvvnmodCode],
    options: Utn57ConversionOptions,
) -> Result<Vec<Utn57WrittenUnit>, Utn57ConversionError> {
    let cleaned = discard_legacy_controls(input);
    let replaced = replace_ir_fina(&cleaned)?;
    let mut decomposed = Vec::with_capacity(replaced.len());
    for &code in &replaced {
        if let Some((_, components)) = ZVVNMOD_CODE_DECOMPOSITIONS
            .iter()
            .find(|(merged, _)| *merged == code)
        {
            decomposed.extend_from_slice(components);
        } else {
            decomposed.push(code);
        }
    }
    let input = decomposed.as_slice();
    let mut output = Vec::new();
    let mut index = 0;
    while index < input.len() {
        let longest = ZVVNMOD_TO_UTN57_MAPPINGS
            .iter()
            .filter(|rule| input[index..].starts_with(rule.sources))
            .map(|rule| rule.sources.len())
            .max()
            .ok_or(Utn57MappingError {
                index,
                code: input[index],
            })?;
        let candidates: Vec<_> = ZVVNMOD_TO_UTN57_MAPPINGS
            .iter()
            .filter(|rule| {
                rule.sources.len() == longest && input[index..].starts_with(rule.sources)
            })
            .collect();
        let first = candidates.first().copied().ok_or(Utn57MappingError {
            index,
            code: input[index],
        })?;
        let aa_candidate = longest == 1 && input[index] == crate::AA_FINA;
        let rule = if aa_candidate {
            let expected = if index == 0 && input.len() == 1 {
                crate::UTN57_AA_ISOL
            } else {
                crate::UTN57_AA_FINA
            };
            candidates
                .iter()
                .copied()
                .find(|rule| rule.targets == [expected])
                .unwrap_or(first)
        } else {
            match options.k_variant {
                Utn57KVariant::K => candidates
                    .iter()
                    .copied()
                    .find(|rule| {
                        rule.targets
                            .iter()
                            .any(|target| target.unit == crate::Utn57Unit::K)
                    })
                    .unwrap_or(first),
                Utn57KVariant::K2 => candidates
                    .iter()
                    .copied()
                    .find(|rule| {
                        rule.targets
                            .iter()
                            .any(|target| target.unit == crate::Utn57Unit::K2)
                    })
                    .unwrap_or(first),
            }
        };
        output.extend_from_slice(rule.targets);
        index += rule.sources.len();
    }
    Ok(output)
}
