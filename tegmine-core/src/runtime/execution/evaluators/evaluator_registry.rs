use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use tegmine_common::prelude::*;

use super::javascript_evaluator::JavascriptEvaluator;
use super::javascript_evaluator_advance::JavascriptEvaluatorAdvance;
// use super::rhai_evaluator::RhaiEvaluator;
use super::value_param_evaluator::ValueParamEvaluator;
use super::Evaluator;

pub struct EvaluatorRegistry;

static REGISTRY: Lazy<DashMap<InlineStr, Box<dyn Evaluator>>> = Lazy::new(|| {
    let map = DashMap::new();
    map.insert(
        InlineStr::from("value-param"),
        Box::new(ValueParamEvaluator) as Box<dyn Evaluator>,
    );
    map.insert(
        InlineStr::from("javascript"),
        Box::new(JavascriptEvaluator) as Box<dyn Evaluator>,
    );
    map.insert(
        InlineStr::from("javascript_advance"),
        Box::new(JavascriptEvaluatorAdvance) as Box<dyn Evaluator>,
    );
    // map.insert(
    //     InlineStr::from("rhai"),
    //     Box::new(RhaiEvaluator) as Box<dyn Evaluator>,
    // );
    map
});

impl EvaluatorRegistry {
    pub fn get_evaluator(evaluator_type: &InlineStr) -> Option<Ref<InlineStr, Box<dyn Evaluator>>> {
        REGISTRY.get(evaluator_type)
    }
}
