use tegmine_common::prelude::*;
use tegmine_common::{TaskType, WorkflowTask};

use super::TaskMapper;
use crate::dao::MetadataDao;
use crate::model::{TaskModel, TaskStatus};
use crate::runtime::execution::mapper::TaskMapperContext;
use crate::utils::ParametersUtils;

/// An implementation of `TaskMapper` to map a `WorkflowTask` of type `TaskType::UserDefined` to a
/// `TaskModel` of type `TaskType::UserDefined` with `TaskStatus::Scheduled`
pub struct UserDefinedTaskMapper;

impl TaskMapper for UserDefinedTaskMapper {
    fn get_task_type(&self) -> &str {
        TaskType::UserDefined.as_ref()
    }

    /// This method maps a `WorkflowTask` of type `TaskType::UserDefined` to a `TaskModel` in a
    /// `TaskStatus::Scheduled` state
    ///
    /// return a List with just one User defined task
    fn get_mapped_tasks(
        &self,
        task_mapper_context: TaskMapperContext,
    ) -> TegResult<Vec<TaskModel>> {
        debug!(
            "TaskMapperContext {:?} in UserDefinedTaskMapper",
            task_mapper_context
        );

        let workflow_task = from_addr_mut!(
            task_mapper_context.workflow_task as *const WorkflowTask as *mut WorkflowTask
        );

        let task_def_guard;
        let task_def = if let Some(task_def) = workflow_task.task_definition.as_ref() {
            task_def
        } else {
            if let Some(task_def_ref) = MetadataDao::get_task_def(&workflow_task.name) {
                task_def_guard = task_def_ref;
                task_def_guard.value()
            } else {
                return fmt_err!(
                    TerminateWorkflow,
                    "Invalid task specified. Cannot find task by name {} in the task definitions",
                    workflow_task.name
                );
            }
        };

        let input = ParametersUtils::get_task_input(
            &workflow_task.input_parameters,
            &task_mapper_context.workflow_model,
            Some(task_def),
            Some(&task_mapper_context.task_id),
        )?;

        let mut user_defined_task = task_mapper_context.create_task_model(TaskStatus::Scheduled);
        user_defined_task.input_data = input;
        user_defined_task.retry_count = task_mapper_context.retry_count;
        user_defined_task.callback_after_seconds = workflow_task.start_delay as i64;
        user_defined_task.rate_limit_per_frequency = task_def.rate_limit_per_frequency.unwrap_or(0);
        user_defined_task.rate_limit_frequency_in_seconds =
            task_def.rate_limit_frequency_in_seconds.unwrap_or(1);
        Ok(vec![user_defined_task])
    }
}
