use tegmine_common::prelude::*;

pub trait Evaluator: Send + Sync {
    /// Evaluate the expression using the inputs provided, if required. Evaluation of the expression
    /// depends on the type of the evaluator.
    ///
    /// Return the evaluation result.
    fn evaluate(&self, expression: &InlineStr, input: &Object) -> TegResult<Object>;
}
