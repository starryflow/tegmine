use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use tegmine_common::prelude::*;

use super::start_workflow::StartWorkflow;
use super::switch::Switch;
use super::terminate::Terminate;
use super::workflow_system_task::WorkflowSystemTask;

/// A container class that holds a mapping of system task types `TaskType` to `WorkflowSystemTask`
/// instances.
pub struct SystemTaskRegistry;

static REGISTRY: Lazy<DashMap<InlineStr, Box<dyn WorkflowSystemTask>>> = Lazy::new(|| {
    let map = DashMap::new();
    map.insert(
        InlineStr::from("StartWorkflow"),
        Box::new(StartWorkflow) as Box<dyn WorkflowSystemTask>,
    );
    map.insert(
        InlineStr::from("Switch"),
        Box::new(Switch) as Box<dyn WorkflowSystemTask>,
    );
    map.insert(
        InlineStr::from("Terminate"),
        Box::new(Terminate) as Box<dyn WorkflowSystemTask>,
    );
    map
});

impl SystemTaskRegistry {
    pub fn get(task_type: &str) -> TegResult<Ref<'static, InlineStr, Box<dyn WorkflowSystemTask>>> {
        REGISTRY
            .get(&InlineStr::from(task_type))
            .ok_or(ErrorCode::IllegalArgument(format!(
                "{} not found in SystemTaskRegistry",
                task_type
            )))
    }

    pub fn is_system_task(task_type: &str) -> bool {
        REGISTRY.contains_key(&InlineStr::from(task_type))
    }
}