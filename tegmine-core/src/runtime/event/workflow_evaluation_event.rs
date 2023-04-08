use crate::model::WorkflowModel;

pub struct WorkflowEvaluationEvent {
    pub workflow_model: WorkflowModel,
}

impl WorkflowEvaluationEvent {
    pub fn new(workflow_model: WorkflowModel) -> Self {
        Self {
            workflow_model: workflow_model,
        }
    }
}
