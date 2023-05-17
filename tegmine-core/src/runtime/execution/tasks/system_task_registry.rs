use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use tegmine_common::prelude::*;
use tegmine_common::TaskType;

use super::set_variable::SetVariable;
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
        TaskType::Switch.as_ref().into(),
        Box::new(Switch) as Box<dyn WorkflowSystemTask>,
    );
    map.insert(
        TaskType::SetVariable.as_ref().into(),
        Box::new(SetVariable) as Box<dyn WorkflowSystemTask>,
    );
    map.insert(
        TaskType::Terminate.as_ref().into(),
        Box::new(Terminate) as Box<dyn WorkflowSystemTask>,
    );
    map.insert(
        TaskType::StartWorkflow.as_ref().into(),
        Box::new(StartWorkflow) as Box<dyn WorkflowSystemTask>,
    );
    map
});

static CUSTOM_REGISTRY: Lazy<DashMap<InlineStr, Box<dyn WorkflowSystemTask>>> =
    Lazy::new(|| DashMap::new());

impl SystemTaskRegistry {
    pub fn get(task_type: &str) -> TegResult<Ref<'static, InlineStr, Box<dyn WorkflowSystemTask>>> {
        let task_type = InlineStr::from(task_type);
        REGISTRY
            .get(&task_type)
            .or(CUSTOM_REGISTRY.get(&task_type))
            .ok_or_else(|| {
                ErrorCode::IllegalArgument(format!("{} not found in SystemTaskRegistry", task_type))
            })
    }

    pub fn is_system_task(task_type: &str) -> bool {
        let task_type = InlineStr::from(task_type);
        REGISTRY.contains_key(&task_type) || CUSTOM_REGISTRY.contains_key(&task_type)
    }

    pub fn register(task_type: &str, task: Box<dyn WorkflowSystemTask>) {
        CUSTOM_REGISTRY.insert(InlineStr::from(task_type), task);
    }

    pub fn unregister(task_type: &str) {
        CUSTOM_REGISTRY.remove(&InlineStr::from(task_type));
    }
}
