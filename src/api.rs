use crate::command_bridge::{convert_zvvnmod_text_with_mongol_norm, MongolNormCommandError};
use std::error::Error;
use std::fmt;

/// Failure while converting ZVVNMOD text to canonical UTN #57 output.
///
/// The concrete normalization backend is intentionally kept behind this
/// backend-neutral error boundary.
#[derive(Debug)]
pub struct Utn57TextConversionError {
    source: MongolNormCommandError,
}

impl fmt::Display for Utn57TextConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.source.fmt(formatter)
    }
}

impl Error for Utn57TextConversionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

impl From<MongolNormCommandError> for Utn57TextConversionError {
    fn from(source: MongolNormCommandError) -> Self {
        Self { source }
    }
}

/// Convert ZVVNMOD text to canonical UTN #57 output.
///
/// This is the stable backend-neutral entry point for both Rust callers and
/// the `zvvnmod-to-utn57` CLI. The normalization implementation may be
/// replaced without changing callers.
pub fn convert_zvvnmod_to_utn57(input: &str) -> Result<String, Utn57TextConversionError> {
    convert_zvvnmod_text_with_mongol_norm(input).map_err(Into::into)
}
