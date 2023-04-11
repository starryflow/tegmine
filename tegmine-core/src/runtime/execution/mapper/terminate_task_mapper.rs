use chrono::Utc;
use tegmine_common::prelude::*;
use tegmine_common::TaskType;

use super::{TaskMapper, TaskMapperContext};
use crate::model::{TaskModel, TaskStatus};
use crate::utils::ParametersUtils;

pub struct TerminateTaskMapper;

impl TaskMapper for TerminateTaskMapper {
    fn get_task_type(&self) -> &str {
        TaskType::Terminate.as_ref()
    }

    fn get_mapped_tasks(
        &self,
        task_mapper_context: TaskMapperContext,
    ) -> TegResult<Vec<TaskModel>> {
        debug!(
            "TaskMapperContext {:?} in TerminateTaskMapper",
            task_mapper_context
        );

        let task_input = ParametersUtils::get_task_input(
            &from_addr!(task_mapper_context.workflow_task).input_parameters,
            task_mapper_context.workflow_model,
            None,
            Some(&task_mapper_context.task_id),
        )?;

        let mut task = task_mapper_context.create_task_model(TaskStatus::InProgress);
        task.task_type = TaskType::Terminate.as_ref().into();
        task.start_time = Utc::now().timestamp_millis();
        task.input_data = task_input;

        Ok(vec![task])
    }
}
