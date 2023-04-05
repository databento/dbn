//! Types for errors that can occur in databento-defs and dependent crates.
use std::{ffi::NulError, fmt::Display, num::TryFromIntError};

/// A simple error type for failed conversions.
#[derive(Debug, Clone)]
pub enum ConversionError {
    /// Received an unexpected `NULL` back from an FFI function.
    NullPointer,
    /// Failed type conversion or casting.
    TypeConversion(&'static str),
}

/// A result of a fallible operation.
pub type Result<T> = std::result::Result<T, ConversionError>;

impl Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::NullPointer => write!(f, "Received unexpected NULL from the FFI"),
            ConversionError::TypeConversion(msg) => write!(f, "Type conversion error: {msg}"),
        }
    }
}

impl std::error::Error for ConversionError {}

impl From<NulError> for ConversionError {
    fn from(_: NulError) -> Self {
        Self::TypeConversion("Missing null byte in CString conversion")
    }
}

impl From<TryFromIntError> for ConversionError {
    fn from(_: TryFromIntError) -> Self {
        Self::TypeConversion("Out of range int conversion")
    }
}
