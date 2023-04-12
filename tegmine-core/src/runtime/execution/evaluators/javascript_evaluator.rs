use tegmine_common::prelude::*;

use super::Evaluator;

pub struct JavascriptEvaluator;

impl Evaluator for JavascriptEvaluator {
    fn evaluate(&self, expression: &InlineStr, input: &Object) -> TegResult<Object> {
        debug!("Javascript evaluator -- expression: {}", expression);
        // Evaluate the expression by using the Javascript evaluation engine.

        // Ok(result) => {
        //     debug!("Javascript evaluator -- result: {}", result);
        //     Ok(result.into())
        // }
        // Err(e) => {
        //     error!(
        //         "Error while evaluating script: {}, error: {:?}",
        //         expression, e
        //     );
        //     fmt_err!(TerminateWorkflow, "{}", e)
        // }
        todo!()
    }
}
