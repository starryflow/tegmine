use chrono::Utc;
use tegmine_common::prelude::*;
use tegmine_common::{TaskType, WorkflowTask};

use super::TaskMapper;
use crate::model::{TaskModel, TaskStatus};
use crate::runtime::execution::mapper::TaskMapperContext;
use crate::runtime::execution::DeciderService;

/// An implementation of `TaskMapper` to map a `WorkflowTask` of type `TaskType::FORK_JOIN` to a
/// LinkedList of `TaskModel` beginning with a completed `TaskType::TASK_TYPE_FORK`, followed by the
/// user defined fork tasks
pub struct ForkJoinTaskMapper;

impl TaskMapper for ForkJoinTaskMapper {
    fn get_task_type(&self) -> &str {
        TaskType::ForkJoin.as_ref()
    }

    /// This method gets the list of tasks that need to scheduled when the task to scheduled is of
    /// type `TaskType::FORK_JOIN`.
    ///
    /// return List of tasks in the following order:
    /// `TaskType#TASK_TYPE_FORK` with `TaskStatus::COMPLETED`
    /// Might be any kind of task, but in most cases is a UserDefinedTask with
    /// `TaskStatus::SCHEDULED`
    fn get_mapped_tasks(
        &self,
        task_mapper_context: TaskMapperContext,
    ) -> TegResult<Vec<TaskModel>> {
        debug!(
            "TaskMapperContext {:?} in ForkJoinTaskMapper",
            task_mapper_context
        );

        let workflow_task = from_addr_mut!(
            task_mapper_context.workflow_task as *const WorkflowTask as *mut WorkflowTask
        );
        let workflow_model = task_mapper_context.workflow_model;

        let mut tasks_to_be_scheduled = Vec::default();
        let mut fork_task = task_mapper_context.create_task_model(TaskStatus::Completed);
        fork_task.task_type = TaskType::TASK_TYPE_FORK.into();
        fork_task.task_def_name = TaskType::TASK_TYPE_FORK.into();
        let epoch_millis = Utc::now().timestamp_millis();
        fork_task.start_time = epoch_millis;
        fork_task.end_time = epoch_millis;
        fork_task.input_data = task_mapper_context.task_input;

        tasks_to_be_scheduled.push(fork_task);
        for tasks in &workflow_task.fork_tasks {
            let task = &tasks[0];
            let task_2 = DeciderService::get_tasks_to_be_scheduled(
                workflow_model,
                task,
                task_mapper_context.retry_count,
            )?;
            tasks_to_be_scheduled.extend(task_2);
        }

        if let Some(join_workflow_task) = workflow_model
            .workflow_definition
            .get_next_task(&workflow_task.task_reference_name)
        {
            if join_workflow_task.type_.eq(TaskType::Join.as_ref()) {
                let join_task = DeciderService::get_tasks_to_be_scheduled(
                    workflow_model,
                    join_workflow_task,
                    task_mapper_context.retry_count,
                )?;
                tasks_to_be_scheduled.extend(join_task);
                return Ok(tasks_to_be_scheduled);
            }
        }
        str_err!(
            TerminateWorkflow,
            "Fork task definition is not followed by a join task.  Check the blueprint"
        )
    }
}
