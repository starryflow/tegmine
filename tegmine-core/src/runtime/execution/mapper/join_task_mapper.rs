use tegmine_common::prelude::*;
use tegmine_common::TaskType;

use super::TaskMapper;
use crate::model::TaskModel;
use crate::runtime::execution::mapper::TaskMapperContext;

/// An implementation of `TaskMapper` to map a `WorkflowTask` of type `TaskType::JOIN` to a
/// `TaskModel` of type `TaskType::JOIN`
pub struct JoinTaskMapper;

impl TaskMapper for JoinTaskMapper {
    fn get_task_type(&self) -> &str {
        TaskType::ForkJoin.as_ref()
    }

    fn get_mapped_tasks(
        &self,
        task_mapper_context: TaskMapperContext,
    ) -> TegResult<Vec<TaskModel>> {
        debug!(
            "TaskMapperContext {:?} in JoinTaskMapper",
            task_mapper_context
        );

        todo!()
    }
}
