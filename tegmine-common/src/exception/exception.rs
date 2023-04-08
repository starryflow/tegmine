use std::backtrace::{Backtrace, BacktraceStatus};
use std::sync::Arc;

use thiserror::Error;

pub type TegResult<T> = std::result::Result<T, ErrorCode>;

#[derive(Error)]
pub struct ErrorCode {
    code: u16,
    display_text: String,
    // cause is only used to contain an `anyhow::Error`.
    // TODO: remove `cause` when we completely get rid of `anyhow::Error`.
    cause: Option<Box<dyn std::error::Error + Sync + Send>>,
    backtrace: Option<ErrorCodeBacktrace>,
}

impl ErrorCode {
    pub fn code(&self) -> u16 {
        self.code
    }

    pub fn message(&self) -> String {
        self.cause
            .as_ref()
            .map(|cause| format!("{}\n{:?}", self.display_text, cause))
            .unwrap_or_else(|| self.display_text.clone())
    }

    pub fn from_std_error<T: std::error::Error>(error: T) -> Self {
        ErrorCode {
            code: 1001, // UnImplement
            display_text: error.to_string(),
            cause: None,
            backtrace: Some(ErrorCodeBacktrace::Origin(Arc::new(Backtrace::capture()))),
        }
    }

    pub fn create(
        code: u16,
        display_text: String,
        cause: Option<Box<dyn std::error::Error + Sync + Send>>,
        backtrace: Option<ErrorCodeBacktrace>,
    ) -> ErrorCode {
        ErrorCode {
            code,
            display_text,
            cause,
            backtrace,
        }
    }
}

impl std::fmt::Debug for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Code: {}, displayText = {}.",
            self.code(),
            self.message(),
        )?;

        match self.backtrace.as_ref() {
            None => Ok(()), // no backtrace
            Some(backtrace) => {
                // TODO: Custom stack frame format for print
                match backtrace {
                    ErrorCodeBacktrace::Origin(backtrace) => {
                        if backtrace.status() == BacktraceStatus::Disabled {
                            write!(f, "\n\n<Backtrace disabled by default. Please use RUST_BACKTRACE=1 to enable> ")
                        } else {
                            write!(f, "\n\n{}", backtrace)
                        }
                    }
                    ErrorCodeBacktrace::Serialized(backtrace) => write!(f, "\n\n{}", backtrace),
                }
            }
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Code: {}, displayText = {}.",
            self.code(),
            self.message(),
        )
    }
}

/// * ErrorCodeBacktrace  **

#[derive(Clone)]
pub enum ErrorCodeBacktrace {
    Serialized(Arc<String>),
    Origin(Arc<Backtrace>),
}

impl ToString for ErrorCodeBacktrace {
    fn to_string(&self) -> String {
        match self {
            ErrorCodeBacktrace::Serialized(backtrace) => Arc::as_ref(backtrace).clone(),
            ErrorCodeBacktrace::Origin(backtrace) => {
                format!("{:?}", backtrace)
            }
        }
    }
}

impl From<&str> for ErrorCodeBacktrace {
    fn from(s: &str) -> Self {
        Self::Serialized(Arc::new(s.to_string()))
    }
}
impl From<String> for ErrorCodeBacktrace {
    fn from(s: String) -> Self {
        Self::Serialized(Arc::new(s))
    }
}

impl From<Arc<String>> for ErrorCodeBacktrace {
    fn from(s: Arc<String>) -> Self {
        Self::Serialized(s)
    }
}

impl From<Backtrace> for ErrorCodeBacktrace {
    fn from(bt: Backtrace) -> Self {
        Self::Origin(Arc::new(bt))
    }
}

impl From<&Backtrace> for ErrorCodeBacktrace {
    fn from(bt: &Backtrace) -> Self {
        Self::Serialized(Arc::new(bt.to_string()))
    }
}

impl From<Arc<Backtrace>> for ErrorCodeBacktrace {
    fn from(bt: Arc<Backtrace>) -> Self {
        Self::Origin(bt)
    }
}
