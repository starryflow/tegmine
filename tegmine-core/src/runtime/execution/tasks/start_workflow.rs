use tegmine_common::TaskType;

use super::workflow_system_task::WorkflowSystemTask;

/// The START_WORKFLOW task starts another workflow. Unlike SUB_WORKFLOW, START_WORKFLOW does not
/// create a relationship between starter and the started workflow. It also does not wait for the
/// started workflow to complete. A START_WORKFLOW is considered successful once the requested
/// workflow is started successfully. In other words, START_WORKFLOW is marked as COMPLETED once the
/// started workflow is in RUNNING state. There is no ability to access the output of the started
/// workflow.
pub struct StartWorkflow;

impl WorkflowSystemTask for StartWorkflow {
    fn get_task_type(&self) -> &str {
        TaskType::StartWorkflow.as_ref()
    }
}
