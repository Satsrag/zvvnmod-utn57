use std::error::Error;
use std::fmt;
use std::path::PathBuf;

const MONGOL_NORM_VERSION: &str = "0.0.4";
const PYTHON_PATH_ENV: &str = "ZVVNMOD_MONGOL_NORM_PATH";

/// Failure to determine a stable installation directory for `mongol-norm`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MongolNormPathError;

impl fmt::Display for MongolNormPathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "set {PYTHON_PATH_ENV}, XDG_DATA_HOME, or HOME to an absolute path to select the mongol-norm install directory"
        )
    }
}

impl Error for MongolNormPathError {}

/// Return the registry-stable `mongol-norm` installation directory.
pub fn mongol_norm_install_path() -> Result<PathBuf, MongolNormPathError> {
    if let Some(path) = std::env::var_os(PYTHON_PATH_ENV).filter(|value| !value.is_empty()) {
        let path = PathBuf::from(path);
        return path
            .is_absolute()
            .then_some(path)
            .ok_or(MongolNormPathError);
    }

    let data_home = std::env::var_os("XDG_DATA_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| {
            std::env::var_os("HOME")
                .filter(|value| !value.is_empty())
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
                .map(|home| home.join(".local/share"))
        })
        .ok_or(MongolNormPathError)?;

    Ok(data_home
        .join("zvvnmod-utn57")
        .join("mongol-norm")
        .join(MONGOL_NORM_VERSION)
        .join("site"))
}
