use chrono::Utc;
use tegmine_common::prelude::*;
use tegmine_common::WorkflowDef;

use super::task_model::TaskModel;
use super::{Workflow, WorkflowStatus};
use crate::runtime::StartWorkflowInput;

#[derive(Clone, Debug)]
pub struct WorkflowModel {
    pub workflow_id: InlineStr,
    pub correlation_id: InlineStr,
    pub priority: i32,
    pub workflow_definition: WorkflowDef,
    pub parent_workflow_id: InlineStr,
    pub parent_workflow_task_id: InlineStr,
    pub tasks: Vec<TaskModel>,
    pub task_to_domain: HashMap<InlineStr, InlineStr>,

    pub event: InlineStr,
    pub variables: HashMap<InlineStr, Object>,
    pub input: HashMap<InlineStr, Object>,
    pub output: HashMap<InlineStr, Object>,
    pub input_payload: HashMap<InlineStr, Object>,
    pub output_payload: HashMap<InlineStr, Object>,
    pub external_input_payload_storage_path: InlineStr,
    pub external_output_payload_storage_path: InlineStr,

    pub status: WorkflowStatus,
    pub previous_status: Option<WorkflowStatus>,
    pub reason_for_incompletion: InlineStr,
    /// Capture the failed taskId if the workflow execution failed because of task failure
    pub failed_task_id: InlineStr,
    pub failed_task_names: HashSet<InlineStr>,
    pub failed_reference_task_names: HashSet<InlineStr>,
    pub re_run_from_workflow_id: InlineStr,
    pub last_retried_time: i64,

    pub owner_app: InlineStr,
    pub create_time: i64,
    pub created_by: InlineStr,
    pub updated_time: i64,
    pub updated_by: InlineStr,
    pub end_time: i64,
}

impl WorkflowModel {
    pub fn new(
        workflow_id: InlineStr,
        workflow_definition: WorkflowDef,
        input: StartWorkflowInput,
    ) -> Self {
        let variables = workflow_definition.variables.clone();
        Self {
            workflow_id,
            correlation_id: input.correlation_id,
            priority: input.priority.unwrap_or(0),
            workflow_definition,
            parent_workflow_id: input.parent_workflow_id,
            parent_workflow_task_id: input.parent_workflow_task_id,
            tasks: Vec::default(),
            task_to_domain: input.task_to_domain,

            event: input.event,
            variables,
            input: HashMap::default(),
            output: HashMap::default(),
            input_payload: HashMap::default(),
            output_payload: HashMap::default(),
            external_input_payload_storage_path: InlineStr::new(),
            external_output_payload_storage_path: InlineStr::new(),

            status: WorkflowStatus::Running,
            previous_status: None,
            reason_for_incompletion: InlineStr::new(),
            failed_task_id: InlineStr::new(),
            failed_task_names: HashSet::default(),
            failed_reference_task_names: HashSet::default(),
            re_run_from_workflow_id: InlineStr::new(),
            last_retried_time: 0,

            owner_app: InlineStr::new(), // WorkflowContext.get().getClientApp()
            create_time: Utc::now().timestamp_millis(),
            created_by: InlineStr::new(),
            updated_time: 0,
            updated_by: InlineStr::new(),
            end_time: 0,
        }
    }

    pub fn has_parent(&self) -> bool {
        !self.parent_workflow_id.trim().is_empty()
    }

    pub fn to_short_string(&self) -> String {
        format!(
            "{}.{}/{}",
            self.workflow_definition.name, self.workflow_definition.version, self.workflow_id
        )
    }

    pub fn get_task_by_ref_name(&self, ref_name: &str) -> TegResult<Option<&TaskModel>> {
        if ref_name.is_empty() {
            return str_err!(UnknownException, "refName passed is null.  Check the workflow execution.  For dynamic tasks, make sure referenceTaskName is set to a not null value");
        }

        let mut found = Vec::default();
        for task in &self.tasks {
            if task.reference_task_name.eq(ref_name) {
                found.push(task);
            }
        }

        Ok(found.pop())
    }

    pub fn to_workflow(self) -> Workflow {
        Workflow { inner: self }
    }
}
