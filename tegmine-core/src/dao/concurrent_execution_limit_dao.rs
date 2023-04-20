use crate::model::TaskModel;

/// support concurrency limits of tasks
pub struct ConcurrentExecutionLimitDao;

impl ConcurrentExecutionLimitDao {
    /// Checks if the number of tasks in progress for the given taskDef will exceed the limit if the
    /// task is scheduled to be in progress (given to the worker or for system tasks start() method
    /// called)
    ///
    /// return true if by executing this task, the limit is breached. false otherwise.
    pub fn exceeds_limit(task: &TaskModel) -> bool {
        // TODO
        false
    }
}
