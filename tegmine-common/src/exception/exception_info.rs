use std::backtrace::Backtrace;
use std::num::{ParseFloatError, ParseIntError};
use std::sync::Arc;

use super::exception::ErrorCodeBacktrace;
use super::ErrorCode;

#[derive(thiserror::Error)]
enum OtherErrors {
    AnyHow { error: anyhow::Error },
}

impl std::fmt::Display for OtherErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OtherErrors::AnyHow { error } => write!(f, "{}", error),
        }
    }
}

impl std::fmt::Debug for OtherErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OtherErrors::AnyHow { error } => write!(f, "{:?}", error),
        }
    }
}

impl From<anyhow::Error> for ErrorCode {
    fn from(error: anyhow::Error) -> Self {
        ErrorCode::create(
            1001,
            format!("{}, source: {:?}", error, error.source()),
            Some(Box::new(OtherErrors::AnyHow { error })),
            Some(ErrorCodeBacktrace::Origin(Arc::new(Backtrace::capture()))),
        )
    }
}

impl From<serde_json::Error> for ErrorCode {
    fn from(error: serde_json::Error) -> Self {
        ErrorCode::from_std_error(error)
    }
}

impl From<ParseIntError> for ErrorCode {
    fn from(error: ParseIntError) -> Self {
        ErrorCode::from_std_error(error)
    }
}

impl From<ParseFloatError> for ErrorCode {
    fn from(error: ParseFloatError) -> Self {
        ErrorCode::from_std_error(error)
    }
}

impl From<std::io::Error> for ErrorCode {
    fn from(error: std::io::Error) -> Self {
        ErrorCode::from_std_error(error)
    }
}

impl From<std::env::VarError> for ErrorCode {
    fn from(error: std::env::VarError) -> Self {
        ErrorCode::from_std_error(error)
    }
}

impl<T> From<crossbeam_channel::SendError<T>> for ErrorCode {
    fn from(error: crossbeam_channel::SendError<T>) -> Self {
        ErrorCode::SendEventFailed(format!("{:?}", error))
    }
}
