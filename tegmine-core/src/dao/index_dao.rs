use crate::model::{TaskSummary, WorkflowSummary};

pub struct IndexDao;

impl IndexDao {
    /// This method should return an unique identifier of the indexed doc
    pub fn index_workflow(_workflow: WorkflowSummary) {}

    /// This method should return an unique identifier of the indexed doc
    pub fn async_index_workflow(_workflow: WorkflowSummary) {}

    pub fn index_task(_task: TaskSummary) {}
}
