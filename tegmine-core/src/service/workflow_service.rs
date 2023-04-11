use tegmine_common::prelude::*;
use tegmine_common::{RerunWorkflowRequest, SkipTaskRequest, StartWorkflowRequest};

use super::ExecutionService;
use crate::model::Workflow;
use crate::runtime::StartWorkflowOperation;
use crate::WorkflowStatus;

pub struct WorkflowService;

impl WorkflowService {
    /// Start a new workflow with StartWorkflowRequest, which allows task to be executed in a
    /// domain.
    ///
    /// return the id of the workflow instance that can be use for tracking.
    pub fn start_workflow(start_workflow_request: StartWorkflowRequest) -> TegResult<InlineStr> {
        StartWorkflowOperation::execute(start_workflow_request.into())
    }

    /// Lists workflows for the given correlation id.

    #[allow(unused)]
    fn get_workflows(
        &self,
        name: &str,
        correlation_id: &str,
        include_closed: bool,
        include_tasks: bool,
    ) -> &[Workflow] {
        unimplemented!()
    }

    /// Gets the workflow by workflow Id.
    pub fn get_execution_status(
        &self,
        workflow_id: &str,
        include_tasks: bool,
    ) -> TegResult<(WorkflowStatus, Option<Workflow>)> {
        ExecutionService::get_execution_status(workflow_id, include_tasks)
    }

    /// Removes the workflow from the system.

    #[allow(unused)]
    fn delete_workflow(&self, workflow_id: &str, archive_workflow: bool) {
        unimplemented!()
    }

    /// Retrieves all the running workflows.
    /// return a list of workflow Ids.

    #[allow(unused)]
    fn get_running_workflows(
        &self,
        workflow_name: &str,
        version: i32,
        start_time: i64,
        end_time: i64,
    ) -> &[&str] {
        unimplemented!()
    }

    /// starts the decision task for workflow.

    #[allow(unused)]
    fn decide_workflow(&self, workflow_id: &str) {
        unimplemented!()
    }

    /// Pauses the workflow given a workflow_id.

    #[allow(unused)]
    fn pause_workflow(&self, workflow_id: &str) {
        unimplemented!()
    }

    /// Resumes the workflow.

    #[allow(unused)]
    fn resume_workflow(&self, workflow_id: &str) {
        unimplemented!()
    }

    /// Skips a given task from a current running workflow.

    #[allow(unused)]
    fn skip_task_from_workflow(
        &self,
        workflow_id: &str,
        task_reference_name: &str,
        skip_task_request: SkipTaskRequest,
    ) {
        unimplemented!()
    }

    /// Reruns the workflow from a specific task.

    #[allow(unused)]
    fn rerun_workflow(&self, workflow_id: &str, request: RerunWorkflowRequest) {
        unimplemented!()
    }

    /// Restarts a completed workflow.

    #[allow(unused)]
    fn restart_workflow(&self, workflow_id: &str, use_latest_definitions: bool) {
        unimplemented!()
    }

    /// Retries the last failed task.

    #[allow(unused)]
    fn retry_workflow(&self, workflow_id: &str, resume_sub_workflow_tasks: bool) {
        unimplemented!()
    }
    /// Resets callback times of all non-terminal SIMPLE tasks to 0.

    #[allow(unused)]
    fn reset_workflow(&self, workflow_id: &str) {
        unimplemented!()
    }

    /// Terminate workflow execution.

    #[allow(unused)]
    fn terminate_workflow(&self, workflow_id: &str, reason: &str) {
        unimplemented!()
    }

    // fn search_workflows

    // fn get_external_storage_location
}
