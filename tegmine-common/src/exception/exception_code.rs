#![allow(non_snake_case)]

use std::backtrace::Backtrace;
use std::sync::Arc;

use super::exception::{ErrorCode, ErrorCodeBacktrace};

macro_rules! build_exceptions {
    ($($body:ident($code:expr)),*$(,)*) => {
            impl ErrorCode {
                $(
                pub fn $body(display_text: impl Into<String>) -> ErrorCode {
                    let bt = Some(ErrorCodeBacktrace::Origin(Arc::new(Backtrace::capture())));
                    ErrorCode::create(
                        $code,
                        display_text.into(),
                        None,
                        bt,
                    )
                }
                paste::item! {
                    pub fn [< $body:snake _ code >] ()  -> u16{
                        $code
                    }

                    pub fn [< $body  Code >] ()  -> u16{
                        $code
                    }
                }
                )*
            }
    }
}

// Internal errors [0, 2000].
impl ErrorCode {
    pub const TERMINATE_WORKFLOW_CODE: u16 = 1006;
}
build_exceptions! {
    Ok(0),
    UnImplement(1001),
    IllegalArgument(1002),
    NotFound(1003),
    Conflict(1004),
    SendEventFailed(1005),
    TerminateWorkflow(ErrorCode::TERMINATE_WORKFLOW_CODE),
    NonTransient(1007),
    ScriptEvalFailed(1008),
    TransientException(1009),
    ExecutionException(1009),
    UnknownException(1999),
}
