use chrono::Utc;
use tegmine_common::prelude::*;
use tegmine_common::{TaskType, WorkflowTask};

use super::{TaskMapper, TaskMapperContext};
use crate::dao::MetadataDao;
use crate::model::{TaskModel, TaskStatus};
use crate::utils::ParametersUtils;

/// An implementation of `TaskMapper` to map a `WorkflowTask` of type `TaskType::DO_WHILE` to a
/// `TaskModel` of type `TaskType::DO_WHILE`
pub struct DoWhileTaskMapper;

impl TaskMapper for DoWhileTaskMapper {
    fn get_task_type(&self) -> &str {
        TaskType::DoWhile.as_ref()
    }

    /// This method maps `TaskMapper` to map a `WorkflowTask` of type `TaskType::DO_WHILE` to a
    /// `TaskModel` of type `TaskType::DO_WHILE` with a status of `TaskStatus::IN_PROGRESS`
    ///
    /// return: A `TaskModel` of type `TaskType::DO_WHILE` in a List
    fn get_mapped_tasks(
        &self,
        task_mapper_context: TaskMapperContext,
    ) -> TegResult<Vec<TaskModel>> {
        debug!(
            "TaskMapperContext {:?} in DoWhileTaskMapper",
            task_mapper_context
        );

        let workflow_task = from_addr_mut!(
            task_mapper_context.workflow_task as *const WorkflowTask as *mut WorkflowTask
        );
        let workflow_model = task_mapper_context.workflow_model;

        if let Some(task) =
            workflow_model.get_task_by_ref_name(&workflow_task.task_reference_name)?
        {
            if task.status.is_terminal() {
                // Since loopTask is already completed no need to schedule task again.
                return Ok(vec![]);
            }
        }

        let task_def_guard;
        let task_def = if let Some(task_def) = workflow_task.task_definition.as_ref() {
            Some(task_def)
        } else {
            if let Some(task_def_ref) = MetadataDao::get_task_def(&workflow_task.name) {
                task_def_guard = task_def_ref;
                Some(task_def_guard.value())
            } else {
                None
            }
        };

        let mut do_while_task = task_mapper_context.create_task_model(TaskStatus::InProgress);
        do_while_task.task_type = TaskType::DoWhile.as_ref().into();
        do_while_task.start_time = Utc::now().timestamp_millis();
        do_while_task.rate_limit_per_frequency = task_def
            .as_ref()
            .and_then(|x| x.rate_limit_per_frequency)
            .unwrap_or(0);
        do_while_task.rate_limit_frequency_in_seconds = task_def
            .as_ref()
            .and_then(|x| x.rate_limit_frequency_in_seconds)
            .unwrap_or(1);
        do_while_task.retry_count = task_mapper_context.retry_count;

        do_while_task.input_data = ParametersUtils::get_task_input(
            &workflow_task.input_parameters,
            workflow_model,
            task_def,
            None,
        )?;

        Ok(vec![do_while_task])
    }
}
