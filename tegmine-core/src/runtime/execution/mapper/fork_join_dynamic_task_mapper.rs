use tegmine_common::prelude::*;
use tegmine_common::TaskType;

use super::TaskMapper;
use crate::model::TaskModel;
use crate::runtime::execution::mapper::TaskMapperContext;

/// An implementation of `TaskMapper` to map a `WorkflowTask` of type `TaskType::FORK_JOIN_DYNAMIC`
/// to a LinkedList of `TaskModel` beginning with a `TaskType::TASK_TYPE_FORK`, followed by the user
/// defined dynamic tasks and a `TaskType::JOIN` at the end
pub struct ForkJoinDynamicTaskMapper;

impl TaskMapper for ForkJoinDynamicTaskMapper {
    fn get_task_type(&self) -> &str {
        TaskType::ForkJoin.as_ref()
    }

    fn get_mapped_tasks(
        &self,
        task_mapper_context: TaskMapperContext,
    ) -> TegResult<Vec<TaskModel>> {
        debug!(
            "TaskMapperContext {:?} in ForkJoinDynamicTaskMapper",
            task_mapper_context
        );

        todo!()
    }
}
