use tegmine_common::TaskType;

use super::workflow_system_task::WorkflowSystemTask;
use crate::model::{TaskModel, TaskStatus, WorkflowModel};

pub struct Switch;

impl WorkflowSystemTask for Switch {
    fn get_task_type(&self) -> &str {
        TaskType::Switch.as_ref()
    }

    fn execute(&self, _workflow: &mut WorkflowModel, task: &mut TaskModel) -> bool {
        task.status = TaskStatus::Completed;
        true
    }
}
