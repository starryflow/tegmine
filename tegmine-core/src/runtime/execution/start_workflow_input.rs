use tegmine_common::prelude::*;
use tegmine_common::{StartWorkflowRequest, WorkflowDef};

pub struct StartWorkflowInput {
    pub name: InlineStr,
    pub version: Option<i32>,
    pub workflow_definition: Option<WorkflowDef>,
    pub workflow_input: HashMap<InlineStr, Object>,
    pub external_input_payload_storage_path: InlineStr,
    pub correlation_id: InlineStr,
    pub priority: Option<i32>,
    pub task_to_domain: HashMap<InlineStr, InlineStr>,

    pub parent_workflow_id: InlineStr,
    pub parent_workflow_task_id: InlineStr,
    pub event: InlineStr,
    pub workflow_id: InlineStr,
    pub triggering_workflow_id: InlineStr,
}

impl From<StartWorkflowRequest> for StartWorkflowInput {
    fn from(request: StartWorkflowRequest) -> Self {
        Self {
            name: request.name,
            version: request.version,
            workflow_definition: request.workflow_def,
            workflow_input: request.input,
            external_input_payload_storage_path: request.external_input_payload_storage_path,
            correlation_id: request.correlation_id,
            priority: Some(request.priority),
            task_to_domain: request.task_to_domain,

            parent_workflow_id: InlineStr::new(),
            parent_workflow_task_id: InlineStr::new(),
            event: InlineStr::new(),
            workflow_id: InlineStr::new(),
            triggering_workflow_id: InlineStr::new(),
        }
    }
}

impl StartWorkflowInput {
    pub fn new(
        name: InlineStr,
        workflow_input: HashMap<InlineStr, Object>,
        correlation_id: InlineStr,
        task_to_domain: HashMap<InlineStr, InlineStr>,
        workflow_id: InlineStr,
        triggering_workflow_id: InlineStr,
    ) -> Self {
        Self {
            name,
            version: None,
            workflow_definition: None,
            workflow_input,
            external_input_payload_storage_path: InlineStr::new(),
            correlation_id,
            priority: None,
            task_to_domain,
            parent_workflow_id: InlineStr::new(),
            parent_workflow_task_id: InlineStr::new(),
            event: InlineStr::new(),
            workflow_id: workflow_id,
            triggering_workflow_id: triggering_workflow_id,
        }
    }
}
