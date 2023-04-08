use tegmine_common::prelude::*;

use super::Evaluator;
use crate::runtime::execution::terminate_workflow_exception;

pub struct ValueParamEvaluator;

impl Evaluator for ValueParamEvaluator {
    fn evaluate(&self, expression: &InlineStr, input: &Object) -> TegResult<Object> {
        debug!("ValueParam evaluator -- evaluating: {}", expression);
        if let Object::Map(input) = input {
            let result = input
                .get(expression)
                .map(|x| x.clone())
                .unwrap_or(Object::Null);
            Ok(result)
        } else {
            error!("Input has to be a Map object: {:?}", input);
            terminate_workflow_exception::STATUS.with(|x| x.take());

            fmt_err!(
                TerminateWorkflow,
                "Input has to be a Map object: {:?}",
                input
            )
        }
    }
}
