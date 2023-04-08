use std::collections::HashMap;

use chrono::Utc;
use tegmine_common::prelude::*;
use tegmine_common::WorkflowTask;

use crate::model::{TaskModel, TaskStatus, WorkflowModel};

/// Business object used for interaction between the DeciderService and Different Mappers
#[derive(Debug)]
pub struct TaskMapperContext<'a> {
    pub workflow_model: &'a WorkflowModel,
    // task_definition: &'a TaskDef,
    pub workflow_task: &'a WorkflowTask,
    pub task_input: HashMap<InlineStr, Object>,
    pub retry_count: i32,
    pub retry_task_id: InlineStr,
    pub task_id: InlineStr,
}

impl<'a> TaskMapperContext<'a> {
    pub fn new(
        workflow_model: &'a WorkflowModel,
        // task_definition: &'a TaskDef,
        workflow_task: &'a WorkflowTask,
        task_input: HashMap<InlineStr, Object>,
        retry_count: i32,
        retry_task_id: InlineStr,
        task_id: InlineStr,
    ) -> Self {
        Self {
            workflow_model,
            // task_definition,
            workflow_task,
            task_input,
            retry_count,
            retry_task_id,
            task_id,
        }
    }

    pub fn create_task_model(&self, status: TaskStatus) -> TaskModel {
        let mut task_model = TaskModel::new(status);
        task_model.reference_task_name = self.workflow_task.task_reference_name.clone();
        task_model.workflow_instance_id = self.workflow_model.workflow_id.clone();
        task_model.workflow_type = self.workflow_model.workflow_definition.name.clone();
        task_model.correlation_id = self.workflow_model.correlation_id.clone();
        task_model.scheduled_time = Utc::now().timestamp_millis();

        task_model.task_id = self.task_id.clone();
        task_model.workflow_task = Some(self.workflow_task.clone());
        task_model.workflow_priority = self.workflow_model.priority;

        task_model.task_type = self.workflow_task.type_.clone();
        task_model.task_def_name = self.workflow_task.name.clone();

        task_model
    }
}
