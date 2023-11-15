//! Types for errors that can occur while working with DBN.
use thiserror::Error;

/// An error that can occur while processing DBN data.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// An I/O error while reading or writing DBN or another encoding.
    #[error("IO error: {source:?} while {context}")]
    Io {
        /// The original error.
        #[source]
        source: std::io::Error,
        /// The context in which the error occurred.
        context: String,
    },
    /// An error while decoding from DBN.
    #[error("decoding error: {0}")]
    Decode(String),
    /// An error with text encoding.
    #[error("encoding error: {0}")]
    Encode(String),
    /// An conversion error between types or encodings.
    #[error("couldn't convert {input} to {desired_type}")]
    Conversion {
        /// The input to the conversion.
        input: String,
        /// The desired type or encoding.
        desired_type: &'static str,
    },
    /// An error with conversion of bytes to UTF-8.
    #[error("UTF-8 error: {source:?} while {context}")]
    Utf8 {
        /// The original error.
        #[source]
        source: std::str::Utf8Error,
        /// The context in which the error occurred.
        context: String,
    },
    /// An invalid argument was passed to a function.
    #[error("bad argument {param_name}: {desc}")]
    BadArgument {
        /// The name of the parameter to which the bad argument was passed.
        param_name: String,
        /// The description of why the argument was invalid.
        desc: String,
    },
}
/// An alias for a `Result` with [`dbn::Error`](crate::Error) as the error type.
pub type Result<T> = std::result::Result<T, Error>;

impl From<csv::Error> for Error {
    fn from(value: csv::Error) -> Self {
        match value.into_kind() {
            csv::ErrorKind::Io(io) => Self::io(io, "while writing CSV"),
            csv::ErrorKind::Utf8 { pos, err } => {
                Self::Encode(format!("UTF-8 error {err:?}{}", Self::opt_pos(&pos)))
            }
            csv::ErrorKind::UnequalLengths {
                pos,
                expected_len,
                len,
            } => Self::Encode(format!(
                "unequal CSV row lengths{}: expected {expected_len}, found {len}",
                Self::opt_pos(&pos)
            )),
            e => Self::Encode(format!("{e:?}")),
        }
    }
}

impl Error {
    /// Creates a new I/O [`dbn::Error`](crate::Error).
    pub fn io(error: std::io::Error, context: impl ToString) -> Self {
        Self::Io {
            source: error,
            context: context.to_string(),
        }
    }

    /// Creates a new decode [`dbn::Error`](crate::Error).
    pub fn decode(msg: impl ToString) -> Self {
        Self::Decode(msg.to_string())
    }

    /// Creates a new encode [`dbn::Error`](crate::Error).
    pub fn encode(msg: impl ToString) -> Self {
        Self::Encode(msg.to_string())
    }

    /// Creates a new conversion [`dbn::Error`](crate::Error) where `desired_type` is `T`.
    pub fn conversion<T>(input: impl ToString) -> Self {
        Self::Conversion {
            input: input.to_string(),
            desired_type: std::any::type_name::<T>(),
        }
    }

    /// Creates a new UTF-8 [`dbn::Error`](crate::Error).
    pub fn utf8(error: std::str::Utf8Error, context: impl ToString) -> Self {
        Self::Utf8 {
            source: error,
            context: context.to_string(),
        }
    }

    fn opt_pos(pos: &Option<csv::Position>) -> String {
        if let Some(pos) = pos.as_ref() {
            format!(" at {pos:?}")
        } else {
            String::default()
        }
    }
}

pub(crate) fn silence_eof_error<T>(err: std::io::Error) -> std::io::Result<Option<T>> {
    if err.kind() == std::io::ErrorKind::UnexpectedEof {
        Ok(None)
    } else {
        Err(err)
    }
}
