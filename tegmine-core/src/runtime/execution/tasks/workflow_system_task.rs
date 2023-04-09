use tegmine_common::prelude::*;

use crate::model::{TaskModel, WorkflowModel};

pub trait WorkflowSystemTask: Send + Sync {
    /// Start the task execution.
    ///
    /// Called only once, and first, when the task status is SCHEDULED.
    fn start(&self, _workflow: &WorkflowModel, _task: &TaskModel) -> TegResult<()> {
        // Do nothing unless overridden by the task implementation
        Ok(())
    }

    /// "Execute" the task.
    /// Called after `start(WorkflowModel, TaskModel, WorkflowExecutor)`, if the task status is not
    /// terminal. Can be called more than once.
    fn execute(&self, _workflow: &WorkflowModel, _task: &mut TaskModel) -> bool {
        false
    }

    /// Cancel task execution
    fn cancel(&self, _workflow: &WorkflowModel, _task: &mut TaskModel) -> TegResult<()> {
        Ok(())
    }

    /// return True if the task is supposed to be started asynchronously using internal queues.
    fn is_async(&self) -> bool {
        false
    }

    /// return True to keep task in 'IN_PROGRESS' state, and 'COMPLETE' later by an external
    /// message.
    fn is_async_complete(&self, task: &TaskModel) -> bool {
        if let Some(async_complete) = task.input_data.get(&InlineStr::from("asyncComplete")) {
            async_complete.as_bool().unwrap_or(false)
        } else {
            task.workflow_task
                .as_ref()
                .map(|x| x.async_complete)
                .unwrap_or(false)
        }
    }

    /// return name of the system task
    fn get_task_type(&self) -> &str;

    /// Default to true for retrieving tasks when retrieving workflow data. Some cases (e.g.
    /// sub_workflows) might not need the tasks at all, and by setting this to false in that case,
    /// you can get a solid performance gain.
    ///
    /// return true for retrieving tasks when getting workflow
    fn is_task_retrieval_required(&self) -> bool {
        true
    }
}
