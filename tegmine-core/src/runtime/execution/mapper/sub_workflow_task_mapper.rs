use tegmine_common::prelude::*;
use tegmine_common::TaskType;

use super::{TaskMapper, TaskMapperContext};
use crate::model::TaskModel;

pub struct SubWorkflowTaskMapper;

impl TaskMapper for SubWorkflowTaskMapper {
    fn get_task_type(&self) -> &str {
        TaskType::SubWorkflow.as_ref()
    }

    fn get_mapped_tasks(
        &self,
        task_mapper_context: TaskMapperContext,
    ) -> TegResult<Vec<TaskModel>> {
        debug!(
            "TaskMapperContext {:?} in SubWorkflowTaskMapper",
            task_mapper_context
        );

        // let workflow_task = from_addr!(task_mapper_context.workflow_task);

        // Check if there are sub workflow parameters, if not throw an exception, cannot initiate a
        // sub-workflow without workflow params

        todo!()
    }
}

impl SubWorkflowTaskMapper {}
