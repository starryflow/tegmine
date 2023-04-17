use tegmine_common::prelude::*;
use tegmine_common::{TaskDef, TaskType, WorkflowTask};

use super::{TaskMapper, TaskMapperContext};
use crate::dao::MetadataDao;
use crate::model::{TaskModel, TaskStatus};
use crate::utils::ParametersUtils;

/// An implementation of `TaskMapper` to map a `WorkflowTask` of type `TaskType::DYNAMIC` to a
/// `TaskModel` based on definition derived from the dynamic task name defined in
/// `WorkflowTask::getInputParameters()`
pub struct DynamicTaskMapper;

impl TaskMapper for DynamicTaskMapper {
    fn get_task_type(&self) -> &str {
        TaskType::Dynamic.as_ref()
    }

    /// This method maps a dynamic task to a `TaskModel` based on the input params
    ///
    /// return A `List` that contains a single `TaskModel` with a `TaskStatus::Scheduled`
    fn get_mapped_tasks(
        &self,
        task_mapper_context: TaskMapperContext,
    ) -> TegResult<Vec<TaskModel>> {
        debug!(
            "TaskMapperContext {:?} in DynamicTaskMapper",
            task_mapper_context
        );

        let workflow_task = from_addr_mut!(
            task_mapper_context.workflow_task as *const WorkflowTask as *mut WorkflowTask
        );

        let task_name = Self::get_dynamic_task_name(
            &task_mapper_context.task_input,
            &workflow_task.dynamic_task_name_param,
        )?;
        workflow_task.name = task_name.clone();

        workflow_task.task_definition = Some(Self::get_dynamic_task_definition(from_addr!(
            task_mapper_context.workflow_task
        ))?);

        let input = ParametersUtils::get_task_input(
            &workflow_task.input_parameters,
            &task_mapper_context.workflow_model,
            workflow_task.task_definition.as_ref(),
            Some(&task_mapper_context.task_id),
        )?;

        // IMPORTANT: The WorkflowTask that is inside TaskMapperContext is changed above
        // createTaskModel() must be called here so the changes are reflected in the created
        // TaskModel

        let mut dynamic_task = task_mapper_context.create_task_model(TaskStatus::Scheduled);
        dynamic_task.start_delay_in_seconds = workflow_task.start_delay;
        dynamic_task.input_data = input;
        dynamic_task.retry_count = task_mapper_context.retry_count;
        dynamic_task.callback_after_seconds = workflow_task.start_delay as i64;
        dynamic_task.response_timeout_seconds = workflow_task
            .task_definition
            .as_ref()
            .expect("not none")
            .response_timeout_seconds as i64;
        dynamic_task.task_type = task_name.clone();
        dynamic_task.retried_task_id = task_mapper_context.retry_task_id.clone();
        dynamic_task.workflow_priority = task_mapper_context.workflow_model.priority;
        Ok(vec![dynamic_task])
    }
}

impl DynamicTaskMapper {
    // Helper method that looks into the input params and returns the dynamic task name
    // return The name of the dynamic task
    fn get_dynamic_task_name<'a>(
        task_input: &'a HashMap<InlineStr, Object>,
        task_name_param: &InlineStr,
    ) -> TegResult<&'a InlineStr> {
        task_input.get(task_name_param).and_then(|x|x.as_string().ok()).ok_or(ErrorCode::TerminateWorkflow(format!("Cannot map a dynamic task based on the parameter and input. Parameter= {}, input= {:?}",task_name_param, task_input)))
    }

    /// This method gets the TaskDefinition for a specific `WorkflowTask`
    /// return An instance of TaskDefinition
    fn get_dynamic_task_definition<'a>(workflow_task: &'a WorkflowTask) -> TegResult<TaskDef> {
        // be moved to DAO
        if let Some(task_def) = workflow_task.task_definition.as_ref() {
            Ok(task_def.clone())
        } else {
            if let Some(task_def) = MetadataDao::get_task_def(&workflow_task.name) {
                Ok(task_def.clone())
            } else {
                fmt_err!(
                    TerminateWorkflow,
                    "Invalid task specified. Cannot find task by name {} in the task definitions",
                    workflow_task.name
                )
            }
        }
    }
}
