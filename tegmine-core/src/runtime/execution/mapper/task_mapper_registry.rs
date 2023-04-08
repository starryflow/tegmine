use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use tegmine_common::prelude::*;
use tegmine_common::TaskType;

use super::start_workflow_task_mapper::StartWorkflowTaskMapper;
use super::switch_task_mapper::SwitchTaskMapper;
use super::terminate_task_mapper::TerminateTaskMapper;
use super::TaskMapper;

static REGISTRY: Lazy<DashMap<InlineStr, Box<dyn TaskMapper>>> = Lazy::new(|| {
    let map = DashMap::new();
    map.insert(
        InlineStr::from("StartWorkflowTaskMapper"),
        Box::new(StartWorkflowTaskMapper) as Box<dyn TaskMapper>,
    );
    map.insert(
        InlineStr::from("SwitchTaskMapper"),
        Box::new(SwitchTaskMapper) as Box<dyn TaskMapper>,
    );
    map.insert(
        InlineStr::from("TerminateTaskMapper"),
        Box::new(TerminateTaskMapper) as Box<dyn TaskMapper>,
    );
    map
});

pub struct TaskMapperRegistry;

impl TaskMapperRegistry {
    pub fn get_task_mapper(typ_: &InlineStr) -> Ref<InlineStr, Box<dyn TaskMapper>> {
        REGISTRY.get(typ_).unwrap_or_else(|| {
            REGISTRY
                .get(TaskType::UserDefined.as_ref().into())
                .expect("USER_DEFINE not none")
        })
    }
}
