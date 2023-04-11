use tegmine_common::prelude::*;
use tegmine_common::TaskType;

use super::{TaskMapper, TaskMapperContext};
use crate::model::{TaskModel, TaskStatus};

pub struct StartWorkflowTaskMapper;

impl TaskMapper for StartWorkflowTaskMapper {
    fn get_task_type(&self) -> &str {
        TaskType::StartWorkflow.as_ref()
    }

    fn get_mapped_tasks(
        &self,
        task_mapper_context: TaskMapperContext,
    ) -> TegResult<Vec<TaskModel>> {
        let mut start_workflow_task = task_mapper_context.create_task_model(TaskStatus::Scheduled);
        start_workflow_task.task_type = TaskType::StartWorkflow.as_ref().into();
        start_workflow_task
            .input_data
            .extend(task_mapper_context.task_input);
        start_workflow_task.callback_after_seconds =
            from_addr!(task_mapper_context.workflow_task).start_delay as i64;
        debug!("{:?} created", start_workflow_task);
        Ok(vec![start_workflow_task])
    }
}
