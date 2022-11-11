use std::{
    convert::Infallible,
    io,
    num::{ParseFloatError, ParseIntError},
};
use thiserror::Error;

/// A reading or parsing error.
#[derive(Error, Debug, Clone, Eq, PartialEq)]
#[non_exhaustive]
#[allow(clippy::module_name_repetitions)]
pub enum AmeError {
    #[error("read error")]
    Io(io::ErrorKind),
    #[error("int parsing error")]
    ParseInt(#[from] ParseIntError),
    #[error("float parsing error")]
    ParseFloat(#[from] ParseFloatError),
    #[error("line too short")]
    TooShortLine,
    #[error("string indexing error")]
    StrIndex,
}

impl From<io::Error> for AmeError {
    fn from(e: io::Error) -> Self {
        Self::Io(e.kind())
    }
}

impl From<io::ErrorKind> for AmeError {
    fn from(k: io::ErrorKind) -> Self {
        Self::Io(k)
    }
}

impl From<Infallible> for AmeError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}
