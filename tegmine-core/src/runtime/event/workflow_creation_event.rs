use crate::runtime::execution::StartWorkflowInput;

pub struct WorkflowCreationEvent {
    pub start_workflow_input: StartWorkflowInput,
}

impl WorkflowCreationEvent {
    pub fn new(start_workflow_input: StartWorkflowInput) -> Self {
        Self {
            start_workflow_input,
        }
    }
}
