use chrono::Utc;
use tegmine_common::prelude::*;
use tegmine_common::{TaskType, WorkflowTask};

use super::TaskMapper;
use crate::model::{TaskModel, TaskStatus};
use crate::runtime::execution::mapper::TaskMapperContext;

/// An implementation of `TaskMapper` to map a `WorkflowTask` of type `TaskType::JOIN` to a
/// `TaskModel` of type `TaskType::JOIN`
pub struct JoinTaskMapper;

impl TaskMapper for JoinTaskMapper {
    fn get_task_type(&self) -> &str {
        TaskType::ForkJoin.as_ref()
    }

    /// This method maps `TaskMapper` to map a `WorkflowTask` of type `TaskType::JOIN` to a
    /// `TaskModel` of type `TaskType::JOIN` with a status of `TaskStatus::IN_PROGRESS`
    ///
    /// return A `TaskModel` of type `TaskType::JOIN` in a List
    fn get_mapped_tasks(
        &self,
        task_mapper_context: TaskMapperContext,
    ) -> TegResult<Vec<TaskModel>> {
        debug!(
            "TaskMapperContext {:?} in JoinTaskMapper",
            task_mapper_context
        );

        let workflow_task = from_addr_mut!(
            task_mapper_context.workflow_task as *const WorkflowTask as *mut WorkflowTask
        );

        let mut join_input = HashMap::new();
        join_input.insert(
            "join_on".into(),
            workflow_task
                .join_on
                .iter()
                .map(|x| x.clone().into())
                .collect::<Vec<Object>>()
                .into(),
        );

        let mut join_task = task_mapper_context.create_task_model(TaskStatus::InProgress);
        join_task.task_type = TaskType::Join.as_ref().into();
        join_task.task_def_name = TaskType::Join.as_ref().into();
        join_task.start_time = Utc::now().timestamp_millis();
        join_task.input_data = join_input;

        Ok(vec![join_task])
    }
}
