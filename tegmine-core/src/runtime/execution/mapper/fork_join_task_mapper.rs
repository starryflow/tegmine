use tegmine_common::prelude::*;
use tegmine_common::TaskType;

use super::TaskMapper;
use crate::model::TaskModel;
use crate::runtime::execution::mapper::TaskMapperContext;

/// An implementation of `TaskMapper` to map a `WorkflowTask` of type `TaskType::FORK_JOIN` to a
/// LinkedList of `TaskModel` beginning with a completed `TaskType::TASK_TYPE_FORK`, followed by the
/// user defined fork tasks
pub struct ForkJoinTaskMapper;

impl TaskMapper for ForkJoinTaskMapper {
    fn get_task_type(&self) -> &str {
        TaskType::ForkJoin.as_ref()
    }

    fn get_mapped_tasks(
        &self,
        task_mapper_context: TaskMapperContext,
    ) -> TegResult<Vec<TaskModel>> {
        debug!(
            "TaskMapperContext {:?} in ForkJoinTaskMapper",
            task_mapper_context
        );

        todo!()
    }
}
