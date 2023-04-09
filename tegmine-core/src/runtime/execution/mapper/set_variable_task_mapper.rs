use chrono::Utc;
use tegmine_common::prelude::*;
use tegmine_common::TaskType;

use super::TaskMapper;
use crate::model::{TaskModel, TaskStatus};
use crate::runtime::execution::mapper::TaskMapperContext;

pub struct SetVariableTaskMapper;

impl TaskMapper for SetVariableTaskMapper {
    fn get_task_type(&self) -> &str {
        TaskType::SetVariable.as_ref()
    }

    fn get_mapped_tasks(
        &self,
        task_mapper_context: TaskMapperContext,
    ) -> TegResult<Vec<TaskModel>> {
        debug!(
            "TaskMapperContext {:?} in SetVariableMapper",
            task_mapper_context
        );

        let mut var_task = task_mapper_context.create_task_model(TaskStatus::InProgress);
        var_task.start_time = Utc::now().timestamp_millis();
        var_task.input_data = task_mapper_context.task_input.clone();

        Ok(vec![var_task])
    }
}
