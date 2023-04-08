use tegmine_common::prelude::*;

use super::task_mapper_context::TaskMapperContext;
use crate::model::TaskModel;

pub trait TaskMapper: Send + Sync {
    fn get_task_type(&self) -> &str;

    fn get_mapped_tasks(&self, task_mapper_context: TaskMapperContext)
        -> TegResult<Vec<TaskModel>>;
}
