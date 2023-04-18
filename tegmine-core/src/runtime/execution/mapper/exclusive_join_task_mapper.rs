use chrono::Utc;
use tegmine_common::prelude::*;
use tegmine_common::TaskType;

use super::TaskMapper;
use crate::model::{TaskModel, TaskStatus};
use crate::runtime::execution::mapper::TaskMapperContext;

pub struct ExclusiveJoinTaskMapper;

impl TaskMapper for ExclusiveJoinTaskMapper {
    fn get_task_type(&self) -> &str {
        TaskType::ExclusiveJoin.as_ref()
    }

    fn get_mapped_tasks(
        &self,
        task_mapper_context: TaskMapperContext,
    ) -> TegResult<Vec<TaskModel>> {
        debug!(
            "TaskMapperContext {:?} in ExclusiveJoinTaskMapper",
            task_mapper_context
        );

        let workflow_task = from_addr!(task_mapper_context.workflow_task);

        let mut join_input = HashMap::default();
        join_input.insert(
            "exclusiveJoinOn".into(),
            workflow_task
                .exclusive_join_on
                .iter()
                .map(|x| x.into())
                .collect::<Vec<Object>>()
                .into(),
        );

        if !workflow_task.default_exclusive_join_task.is_empty() {
            join_input.insert(
                "defaultExclusiveJoinTask".into(),
                workflow_task
                    .default_exclusive_join_task
                    .iter()
                    .map(|x| x.into())
                    .collect::<Vec<Object>>()
                    .into(),
            );
        }

        let mut join_task = task_mapper_context.create_task_model(TaskStatus::InProgress);
        join_task.task_type = TaskType::ExclusiveJoin.as_ref().into();
        join_task.task_def_name = TaskType::ExclusiveJoin.as_ref().into();
        join_task.start_time = Utc::now().timestamp_millis();
        join_task.input_data = join_input;

        Ok(vec![join_task])
    }
}
