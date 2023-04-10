use tegmine_common::prelude::*;

use super::Evaluator;

pub struct ValueParamEvaluator;

impl Evaluator for ValueParamEvaluator {
    fn evaluate(&self, expression: &InlineStr, input: &Object) -> TegResult<Object> {
        debug!(
            "ValueParam evaluator -- evaluating: {} with input: {:?}",
            expression, input
        );
        if let Object::Map(input) = input {
            let result = input
                .get(expression)
                .map(|x| x.clone())
                .unwrap_or(Object::Null);
            debug!("ValueParam evaluator -- result is: {:?}", result);
            Ok(result)
        } else {
            error!("Input has to be a Map object: {:?}", input);
            fmt_err!(
                TerminateWorkflow,
                "Input has to be a Map object: {:?}",
                input
            )
        }
    }
}
