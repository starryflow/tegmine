use tegmine_common::prelude::*;

use crate::model::TaskStatus;
use crate::WorkflowStatus;

pub struct Monitors;

impl Monitors {
    pub fn error(class_name: &str, method_name: &str) {}

    pub fn record_workflow_decision_time(duration: i64) {}

    pub fn record_workflow_completion(workflow_type: &str, duration: i64, owner_app: &str) {}

    pub fn record_workflow_termination(
        workflow_type: &str,
        status: WorkflowStatus,
        owner_app: &str,
    ) {
    }

    pub fn record_workflow_start_error(workflow_type: &str, owner_app: &str) {}

    pub fn record_update_conflict(task_type: &str, workflow_type: &str, status: TaskStatus) {}

    pub fn record_task_queue_op_error(task_type: &str, workflow_type: &str) {}

    pub fn record_task_update_error(task_type: &str, workflow_type: &str) {}

    pub fn record_task_execution_time(
        task_type: &str,
        duration: i64,
        includes_retries: bool,
        status: TaskStatus,
    ) {
    }

    pub fn record_task_extend_lease_error(task_type: &str, workflow_type: &str) {}

    pub fn record_num_tasks_in_workflow(count: i64, name: &str, version: &str) {}

    pub fn record_acquire_lock_unsuccessful() {}
}
