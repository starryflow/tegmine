use super::WorkflowModel;

/// Captures workflow summary info to be indexed in Elastic Search.
pub struct WorkflowSummary;

impl WorkflowSummary {
    pub fn new(_workflow: &WorkflowModel) -> Self {
        Self {}
    }
}
