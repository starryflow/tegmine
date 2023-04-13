use tegmine_common::prelude::*;

use super::Evaluator;
use crate::utils::RuntimeEngine;

pub struct JavascriptEvaluator;

impl Evaluator for JavascriptEvaluator {
    fn evaluate(&self, expression: &InlineStr, input: &Object) -> TegResult<Object> {
        debug!("Javascript evaluator -- expression: {}", expression);
        // Evaluate the expression by using the Javascript evaluation engine.

        let ref mut isolate = v8::Isolate::new(Default::default());
        let ref mut isolated_scope = v8::HandleScope::new(isolate);
        let mut rt_engine = RuntimeEngine::new(
            &format!("function process($) {{ return {} }}", expression),
            "process",
            isolated_scope,
        );
        match rt_engine.call_one_arg(input) {
            Ok(result) => {
                debug!("Javascript evaluator -- result: {:?}", result);
                Ok(result)
            }
            Err(e) => {
                error!(
                    "Error while evaluating script: {}, error: {:?}",
                    expression, e
                );
                fmt_err!(TerminateWorkflow, "{}", e)
            }
        }
    }
}
