use tegmine_common::prelude::*;
use tegmine_common::{TaskType, WorkflowTask};

use super::TaskMapper;
use crate::model::{TaskModel, TaskStatus};
use crate::runtime::execution::mapper::TaskMapperContext;
use crate::utils::ParametersUtils;

/// An implementation of `TaskMapper` to map a `WorkflowTask` of type `TaskType::SIMPLE` to a
/// `TaskModel` with status `TaskModel.Status::SCHEDULED`. NOTE: There is not type defined for
/// simples task.
pub struct SimpleTaskMapper;

impl TaskMapper for SimpleTaskMapper {
    fn get_task_type(&self) -> &str {
        TaskType::ForkJoin.as_ref()
    }

    /// This method maps a `WorkflowTask` of type `TaskType::SIMPLE` to a `TaskModel`
    ///
    /// return a List with just one simple task
    fn get_mapped_tasks(
        &self,
        task_mapper_context: TaskMapperContext,
    ) -> TegResult<Vec<TaskModel>> {
        debug!(
            "TaskMapperContext {:?} in SimpleTaskMapper",
            task_mapper_context
        );

        let workflow_task = from_addr_mut!(
            task_mapper_context.workflow_task as *const WorkflowTask as *mut WorkflowTask
        );

        let task_def = if let Some(task_def) = workflow_task.task_definition.as_ref() {
            task_def
        } else {
            return fmt_err!(
                TerminateWorkflow,
                "Invalid task. Task {} does not have a definition",
                workflow_task.name
            );
        };

        let input = ParametersUtils::get_task_input(
            &workflow_task.input_parameters,
            &task_mapper_context.workflow_model,
            Some(task_def),
            Some(&task_mapper_context.task_id),
        )?;

        let mut simple_task = task_mapper_context.create_task_model(TaskStatus::Scheduled);
        simple_task.task_type = workflow_task.name.clone();
        simple_task.start_delay_in_seconds = workflow_task.start_delay;
        simple_task.input_data = input;
        simple_task.retry_count = task_mapper_context.retry_count;
        simple_task.callback_after_seconds = workflow_task.start_delay as i64;
        simple_task.response_timeout_seconds = task_def.get_response_timeout_seconds() as i64;
        simple_task.retried_task_id = task_mapper_context.retry_task_id;
        simple_task.rate_limit_per_frequency = task_def.rate_limit_per_frequency.unwrap_or(0);
        simple_task.rate_limit_frequency_in_seconds =
            task_def.rate_limit_frequency_in_seconds.unwrap_or(1);

        Ok(vec![simple_task])
    }
}
