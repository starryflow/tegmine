use chrono::Utc;
use tegmine_common::prelude::*;

use crate::dao::{MetadataDao, QueueDao};
use crate::metrics::Monitors;
use crate::runtime::{ExecutionDaoFacade, WorkflowExecutor};
use crate::utils::QueueUtils;
use crate::{TaskModel, TaskStatus, WorkflowSystemTask};

const QUEUE_TASK_MESSAGE_POSTPONE_SECS: i64 = 60;
const SYSTEM_TASK_CALLBACK_TIME: i64 = 30;

pub struct AsyncSystemTaskExecutor;

impl AsyncSystemTaskExecutor {
    pub fn execute(
        system_task: Arc<Box<dyn WorkflowSystemTask>>,
        task_id: &InlineStr,
    ) -> TegResult<()> {
        fn _execute(
            task: &mut TaskModel,
            workflow_id: &InlineStr,
            system_task: Arc<Box<dyn WorkflowSystemTask>>,
            queue_name: &str,
            has_task_execution_completed: &mut bool,
        ) -> TegResult<()> {
            let mut workflow = ExecutionDaoFacade::get_workflow_model(
                &workflow_id,
                system_task.is_task_retrieval_required(),
            )?;

            if workflow.status.is_terminal() {
                info!(
                    "Workflow {} has been completed for {}/{}",
                    workflow.to_short_string(),
                    system_task.get_task_type(),
                    task.task_id
                );
                if !task.status.is_terminal() {
                    task.status = TaskStatus::Canceled;
                    task.reason_for_incompletion =
                        InlineStr::from(format!("Workflow is in {:?} state", workflow.status));
                }
                QueueDao::remove(&queue_name, &task.task_id)?;
                return Ok(());
            }

            debug!(
                "Executing {}/{} in {:?} state",
                task.task_type, task.task_id, task.status
            );

            let is_task_async_complete = system_task.is_async_complete(&task);
            if task.status == TaskStatus::Scheduled || !is_task_async_complete {
                task.poll_count += 1;
            }

            if task.status == TaskStatus::Scheduled {
                task.start_time = Utc::now().timestamp_millis();
                Monitors::record_queue_wait_time(&task.task_type, task.get_queue_wait_time());
                system_task.start(&mut workflow, task)?;
            } else if task.status == TaskStatus::InProgress {
                system_task.execute(&mut workflow, task);
            }

            // Update message in Task queue based on Task status
            // Remove asyncComplete system tasks from the queue that are not in SCHEDULED state
            if is_task_async_complete && task.status != TaskStatus::Scheduled {
                QueueDao::remove(queue_name, &task.task_id)?;
                *has_task_execution_completed = true;
            } else if task.status.is_terminal() {
                task.end_time = Utc::now().timestamp_millis();
                QueueDao::remove(queue_name, &task.task_id)?;
                *has_task_execution_completed = true;
                debug!("{:?} removed from queue: {}", task, queue_name);
            } else {
                task.callback_after_seconds = SYSTEM_TASK_CALLBACK_TIME;
                QueueDao::postpone(
                    queue_name,
                    &task.task_id,
                    task.workflow_priority,
                    SYSTEM_TASK_CALLBACK_TIME,
                )?;
                debug!("{:?} postponed in queue: {}", task, queue_name);
            }

            debug!(
                "Finished execution of {}/{}-{:?}",
                system_task.get_task_type(),
                task.task_id,
                task.status
            );

            Ok(())
        }

        let task = Self::load_task_quietly(task_id);
        if task.is_none() {
            error!(
                "TaskId: {} could not be found while executing {}",
                task_id,
                system_task.get_task_type()
            );
            return Ok(());
        }
        let mut task = task.expect("not none");

        debug!(
            "Task: {:?} fetched from execution Dao for taskId: {}",
            task, task_id,
        );
        let queue_name = QueueUtils::get_queue_name_by_task_model(&task);

        if task.status.is_terminal() {
            // Tune the SystemTaskWorkerCoordinator's queues - if the queue size is very big this
            // can happen!
            info!(
                "Task {}/{} was already completed.",
                task.task_type, task.task_id
            );
            QueueDao::remove(&queue_name, &task.task_id)?;
            return Ok(());
        }

        if task.status == TaskStatus::Scheduled {
            if ExecutionDaoFacade::exceeds_in_progress_limit(&task) {
                warn!(
                    "Concurrent Execution limited for {}:{}",
                    task_id, task.task_def_name
                );
                Self::postpone_quietly(&queue_name, &task);
                return Ok(());
            }

            let task_def_guard = MetadataDao::get_task_def(&task.task_def_name);
            let task_def = task_def_guard.as_ref().map(|x| x.value());

            if task.rate_limit_per_frequency > 0
                && ExecutionDaoFacade::exceeds_rate_limit_per_frequency(&task, task_def)
            {
                warn!(
                    "RateLimit Execution limited for {}:{}, limit:{}",
                    task_id, task.task_def_name, task.rate_limit_per_frequency
                );
                Self::postpone_quietly(&queue_name, &task);
                return Ok(());
            }
        }

        let workflow_id = task.workflow_instance_id.clone();
        // if we are here the Task object is updated and needs to be persisted regardless of an
        // exception

        let mut has_task_execution_completed = false;
        let task_type = system_task.get_task_type().to_string();
        if let Err(e) = _execute(
            &mut task,
            &workflow_id,
            system_task,
            &queue_name,
            &mut has_task_execution_completed,
        ) {
            Monitors::error("AsyncSystemTaskExecutor", "executeSystemTask");

            error!(
                "Error executing system task - {}, with id: {} {}",
                task_type, task_id, e
            );
        };

        // } finally {
        ExecutionDaoFacade::update_task(&mut task)?;
        // if the current task execution has completed, then the workflow needs to be evaluated
        if has_task_execution_completed {
            WorkflowExecutor::decide_workflow_id(&workflow_id)?;
        }

        Ok(())
    }

    fn postpone_quietly(queue_name: &str, task: &TaskModel) {
        if QueueDao::postpone(
            queue_name,
            &task.task_id,
            task.workflow_priority,
            QUEUE_TASK_MESSAGE_POSTPONE_SECS,
        )
        .is_err()
        {
            error!(
                "Error postponing task: {} in queue: {}",
                &task.task_id, queue_name
            );
        }
    }

    fn load_task_quietly(task_id: &InlineStr) -> Option<TaskModel> {
        ExecutionDaoFacade::get_task_model(task_id)
    }
}
