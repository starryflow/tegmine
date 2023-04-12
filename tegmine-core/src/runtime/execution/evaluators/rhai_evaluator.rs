use either::Either;
use rhai::{Engine, Scope};
use tegmine_common::prelude::*;

use super::Evaluator;

pub struct RhaiEvaluator;

impl Evaluator for RhaiEvaluator {
    fn evaluate(&self, expression: &InlineStr, input: &Object) -> TegResult<Object> {
        debug!("Rhai evaluator -- expression: {}", expression);
        // Evaluate the expression by using the Javascript evaluation engine.

        let mut engine = Engine::new_raw();
        let mut scope = Scope::new();

        let input_value = Object::read(&mut Either::Right(input.to_json()), "$.inputValue");
        // scope.push_constant("$.inputValue", input_value);
        // scope.push_constant("$.inputValue", "fedex");
        scope.push_constant("inputValue", "fedex");

        // for example: "$.inputValue == 'fedex' ? 'fedex' : 'ups'"
        match engine.eval_expression_with_scope::<InlineStr>(
            &mut scope,
            r#"
                if inputValue == "fedex" {"fedex"}  else {"ups"}
            "#,
        ) {
            Ok(result) => {
                debug!("Rhai evaluator -- result: {}", result);
                Ok(result.into())
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
