use tegmine_common::prelude::*;
use tegmine_common::TaskResult;

use crate::model::Task;
use crate::ExecutionService;

pub struct TaskService;

impl TaskService {
    /// Batch Poll for a task of a certain type.
    pub fn batch_poll(
        task_type: &str,
        worker_id: &str,
        domain: &str,
        count: i32,
        timeout: i32,
    ) -> TegResult<Vec<Task>> {
        let polled_tasks = ExecutionService::poll(task_type, worker_id, domain, count, timeout)?;
        debug!(
            "The Tasks {:?} being returned for /tasks/poll/{}?{}&{}",
            polled_tasks
                .iter()
                .map(|x| x.inner.task_id.clone())
                .collect::<Vec<_>>(),
            task_type,
            worker_id,
            domain
        );
        // Monitors.recordTaskPollCount(taskType, domain, polledTasks.size());
        Ok(polled_tasks)
    }

    /// Updates a task.
    ///
    /// return task Id of the updated task.
    pub fn update_task(task_result: TaskResult) -> String {
        // debug!(
        //     "Update Task: {} with callback time:
        // {}",
        //     task_result, task_result.callback_after_seconds
        // );
        // ExecutionService::update_task(task_result);
        // debug!("Task: {} updated successfully with callback time: {}",
        // task_result,task_result.callback_after_seconds); task_result.task_id
        todo!()
    }
}
