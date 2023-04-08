use super::Workflow;

/// Captures workflow summary info to be indexed in Elastic Search.
pub struct WorkflowSummary;

impl WorkflowSummary {
    pub fn new(_workflow: Workflow) -> Self {
        Self {}
    }
}
