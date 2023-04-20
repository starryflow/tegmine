use chrono::Utc;
use tegmine_common::prelude::*;
use tegmine_common::TaskResult;

use crate::config::Properties;
use crate::dao::QueueDao;
use crate::model::{Task, TaskStatus, Workflow};
use crate::runtime::{ExecutionDaoFacade, WorkflowExecutor};
use crate::utils::QueueUtils;
use crate::WorkflowStatus;

pub struct ExecutionService;

impl ExecutionService {
    const MAX_POLL_TIMEOUT_MS: i32 = 5000;

    pub fn poll(
        task_type: &str,
        worker_id: &str,
        domain: &str,
        count: i32,
        timeout_millis: i32,
    ) -> TegResult<Vec<Task>> {
        if timeout_millis > Self::MAX_POLL_TIMEOUT_MS {
            return str_err!(
                IllegalArgument,
                "Long Poll Timeout value cannot be more than 5 seconds"
            );
        }
        let queue_name = QueueUtils::get_queue_name(
            &task_type.into(),
            &domain.into(),
            &InlineStr::new(),
            &InlineStr::new(),
        );

        let mut tasks = Vec::default();
        for task_id in QueueDao::pop(&queue_name, count, timeout_millis).unwrap_or_else(|e| {
            error!(
                "Error polling for task: {} from worker: {} in domain: {}, count: {}, error: {:?}",
                task_type, worker_id, domain, count, e
            );
            // Monitors.error(this.getClass().getCanonicalName(), "taskPoll");
            // Monitors.recordTaskPollError(taskType, domain, e.getClass().getSimpleName());
            Vec::default()
        }) {
            let task_model = ExecutionDaoFacade::get_task_model(&task_id);
            if task_model.is_none() || task_model.as_ref().expect("not none").status.is_terminal() {
                // Remove taskId(s) without a valid Task/terminal state task from the queue
                if let Err(e) = QueueDao::remove(&queue_name, &task_id) {
                    catch(e, &queue_name, &task_id);
                } else {
                    debug!("Removed task: {} from the queue: {}", task_id, queue_name);
                }
                continue;
            }
            let mut task_model = task_model.expect("not none");

            if ExecutionDaoFacade::exceeds_in_progress_limit(&task_model) {
                // Postpone this message, so that it would be available for poll again.
                if let Err(e) = QueueDao::postpone(
                    &queue_name,
                    &task_id,
                    task_model.workflow_priority,
                    Properties::default().task_execution_postpone_duration_sec,
                ) {
                    catch(e, &queue_name, &task_id);
                } else {
                    debug!(
                        "Postponed task: {} in queue: {} by {} seconds",
                        task_id,
                        queue_name,
                        Properties::default().task_execution_postpone_duration_sec
                    );
                }
                continue;
            }

            let task_def = task_model.get_task_definition();
            if task_model.rate_limit_per_frequency > 0
                && ExecutionDaoFacade::exceeds_rate_limit_per_frequency(&task_model, task_def)
            {
                // Postpone this message, so that it would be available for poll again.
                if let Err(e) = QueueDao::postpone(
                    &queue_name,
                    &task_id,
                    task_model.workflow_priority,
                    Properties::default().task_execution_postpone_duration_sec,
                ) {
                    catch(e, &queue_name, &task_id);
                } else {
                    debug!(
                        "RateLimit Execution limited for {}:{}, limit:{}",
                        task_id, task_model.task_def_name, task_model.rate_limit_per_frequency
                    );
                }
                continue;
            }

            task_model.status = TaskStatus::InProgress;
            if task_model.start_time == 0 {
                task_model.start_time = Utc::now().timestamp_millis();
                // Monitors.recordQueueWaitTime(
                //             taskModel.getTaskDefName(), taskModel.getQueueWaitTime());
            }
            // reset callbackAfterSeconds when giving the task to the worker
            task_model.callback_after_seconds = 0;
            task_model.worker_id = worker_id.into();
            task_model.poll_count += 1;
            if let Err(e) = ExecutionDaoFacade::update_task(&mut task_model) {
                catch(e, &queue_name, &task_id);
            } else {
                tasks.push(task_model.to_task());
            }

            fn catch(e: ErrorCode, queue_name: &InlineStr, task_id: &InlineStr) {
                warn!(
                    "DB operation failed for task: {}, postponing task in queue, error: {}",
                    task_id, e
                );
                // Monitors.recordTaskPollError(taskType, domain, e.getClass().getSimpleName());
                let _ = QueueDao::postpone(
                    queue_name,
                    &task_id,
                    0,
                    Properties::default().task_execution_postpone_duration_sec,
                );
            }
        }

        ExecutionDaoFacade::update_task_last_poll(task_type, domain, worker_id);
        // Monitors.recordTaskPoll(queueName);
        tasks.iter().for_each(|x| {
            let _ = Self::ack_task_received(x);
        });
        Ok(tasks)
    }

    pub fn update_task(task_result: TaskResult) -> TegResult<()> {
        WorkflowExecutor::update_task(task_result)
    }

    pub fn ack_task_received(task: &Task) -> bool {
        QueueDao::ack(
            &QueueUtils::get_queue_name_by_task_model(&task.inner),
            &task.inner.task_id,
        )
    }

    pub fn get_execution_status(
        workflow_id: &str,
        include_tasks: bool,
    ) -> TegResult<(WorkflowStatus, Option<Workflow>)> {
        if let Some(status) = ExecutionDaoFacade::get_workflow_status(&workflow_id.into()) {
            if status.is_terminal() {
                Ok((
                    status,
                    Some(ExecutionDaoFacade::get_workflow(
                        &workflow_id.into(),
                        include_tasks,
                    )?),
                ))
            } else {
                Ok((status, None))
            }
        } else {
            fmt_err!(NotFound, "can not find workflow: {}", workflow_id)
        }
    }
}
