use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use tegmine_common::prelude::*;

use super::value_param_evaluator::ValueParamEvaluator;
use super::Evaluator;

pub struct EvaluatorRegistry;

static REGISTRY: Lazy<DashMap<InlineStr, Box<dyn Evaluator>>> = Lazy::new(|| {
    let map = DashMap::new();
    map.insert(
        InlineStr::from("ValueParamEvaluator"),
        Box::new(ValueParamEvaluator) as Box<dyn Evaluator>,
    );
    map
});

impl EvaluatorRegistry {
    pub fn get_evaluator(evaluator_type: &InlineStr) -> Option<Ref<InlineStr, Box<dyn Evaluator>>> {
        REGISTRY.get(evaluator_type)
    }
}
