use chrono::Utc;
use tegmine_common::prelude::*;
use tegmine_common::TaskType;

use super::{TaskMapper, TaskMapperContext};
use crate::model::{TaskModel, TaskStatus};
use crate::runtime::execution::evaluators::EvaluatorRegistry;
use crate::runtime::execution::DeciderService;

/// An implementation of `TaskMapper` to map a `WorkflowTask` of type `TaskType::Switch` to a List
/// `TaskModel` starting with Task of type `TaskType::Switch` which is marked as InProgress,
/// followed by the list of `TaskModel` based on the case expression evaluation in the Switch task.
pub struct SwitchTaskMapper;

impl TaskMapper for SwitchTaskMapper {
    fn get_task_type(&self) -> &str {
        TaskType::Switch.as_ref()
    }

    /// This method gets the list of tasks that need to scheduled when the task to scheduled is of
    /// type `TaskType::Switch`}.
    ///
    /// return List of tasks in the following order:
    /// - `TaskType::Switch` with `TaskStatus::InProgress`
    /// - `List of tasks based on the evaluation of `WorkflowTask::getEvaluatorType()` and
    /// `WorkflowTask::getExpression()` are scheduled.     
    /// - `In the case of no matching `WorkflowTask::getEvaluatorType()`, workflow will be
    ///   terminated with error message. In case of no matching result after the evaluation of the
    ///   `WorkflowTask::getExpression()`, the `WorkflowTask::getDefaultCase()` Tasks are scheduled.
    fn get_mapped_tasks(
        &self,
        mut task_mapper_context: TaskMapperContext,
    ) -> TegResult<Vec<TaskModel>> {
        debug!(
            "TaskMapperContext {:?} in TerminateTaskMapper",
            task_mapper_context
        );
        let mut tasks_to_be_scheduled = Vec::default();

        let workflow_task = from_addr!(task_mapper_context.workflow_task);
        let task_input = std::mem::take(&mut task_mapper_context.task_input).into();

        // get the expression to be evaluated
        let evaluator = EvaluatorRegistry::get_evaluator(&workflow_task.evaluator_type);
        if evaluator.is_none() {
            error!(
                "No evaluator registered for type: {}",
                workflow_task.evaluator_type
            );
            return fmt_err!(
                TerminateWorkflow,
                "No evaluator registered for type: {}",
                workflow_task.evaluator_type
            );
        }
        let eval_result = evaluator
            .expect("not none")
            .evaluate(&workflow_task.expression, &task_input)?
            .as_string()?
            .clone();
        debug!("eval_result is: {}", eval_result);

        let mut switch_task = task_mapper_context.create_task_model(TaskStatus::InProgress);
        switch_task.task_type = TaskType::Switch.as_ref().into();
        switch_task.task_def_name = TaskType::Switch.as_ref().into();
        switch_task
            .input_data
            .insert("case".into(), eval_result.clone().into());
        switch_task.output_data.insert(
            "evaluationResult".into(),
            vec![eval_result.clone().into()].into(),
        );
        switch_task.start_time = Utc::now().timestamp_millis();
        tasks_to_be_scheduled.push(switch_task);

        // get the list of tasks based on the evaluated expression
        debug!("decision_case is: {:?}", workflow_task.decision_cases);
        let selected_tasks =
            if let Some(selected_task) = workflow_task.decision_cases.get(&eval_result) {
                if !selected_task.is_empty() {
                    selected_task
                } else {
                    &workflow_task.default_case
                }
            } else {
                // if the tasks returned are empty based on evaluated result, then get the default
                // case if there is one
                &workflow_task.default_case
            };

        // once there are selected tasks that need to proceeded as part of the switch, get the next
        // task to be scheduled by using the decider service
        if !selected_tasks.is_empty() {
            // Schedule the first task to be executed...
            let selected_task = &selected_tasks[0];
            // TODO break out this recursive call using function composition of what needs to be
            // done and then walk back the condition tree
            let case_tasks = DeciderService::get_tasks_to_be_scheduled_with_retry(
                task_mapper_context.workflow_model,
                selected_task,
                task_mapper_context.retry_count,
                &task_mapper_context.retry_task_id,
            )?;
            tasks_to_be_scheduled.extend(case_tasks);
            tasks_to_be_scheduled[0]
                .input_data
                .insert("hasChildren".into(), "true".into());
        }

        Ok(tasks_to_be_scheduled)
    }
}
