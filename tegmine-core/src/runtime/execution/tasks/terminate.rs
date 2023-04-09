use tegmine_common::prelude::*;
use tegmine_common::TaskType;

use super::workflow_system_task::WorkflowSystemTask;
use crate::model::{TaskModel, TaskStatus, WorkflowModel};
use crate::WorkflowStatus;

/// Task that can terminate a workflow with a given status and modify the workflow's output with a
/// given parameter, it can act as a "return" statement for conditions where you simply want to
/// terminate your workflow. For example, if you have a decision where the first condition is met,
/// you want to execute some tasks, otherwise you want to finish your workflow.
pub struct Terminate;

impl WorkflowSystemTask for Terminate {
    fn get_task_type(&self) -> &str {
        TaskType::Terminate.as_ref()
    }

    fn execute(&self, _workflow: &WorkflowModel, task: &mut TaskModel) -> bool {
        let return_status = task
            .input_data
            .get(Self::TERMINATION_STATUS_PARAMETER)
            .and_then(|x| x.as_string().ok());

        if Self::validate_input_status(return_status) {
            task.output_data = Self::get_input_from_param(&task.input_data);
            task.status = TaskStatus::Completed;
            true
        } else {
            task.reason_for_incompletion = "given termination status is not valid".into();
            task.status = TaskStatus::Failed;
            false
        }
    }
}

impl Terminate {
    const TERMINATION_STATUS_PARAMETER: &'static str = "terminationStatus";
    const TERMINATION_REASON_PARAMETER: &'static str = "terminationReason";
    const TERMINATION_WORKFLOW_OUTPUT: &'static str = "workflowOutput";

    pub fn get_termination_status_parameter() -> InlineStr {
        Self::TERMINATION_STATUS_PARAMETER.into()
    }
    pub fn get_termination_reason_parameter() -> InlineStr {
        Self::TERMINATION_REASON_PARAMETER.into()
    }

    fn validate_input_status(status: Option<&InlineStr>) -> bool {
        status
            .map(|x| {
                x.eq(WorkflowStatus::Completed.as_ref()) || x.eq(WorkflowStatus::Failed.as_ref())
            })
            .unwrap_or(false)
    }

    fn get_input_from_param(task_input: &HashMap<InlineStr, Object>) -> HashMap<InlineStr, Object> {
        let mut output = HashMap::default();
        if let Some(input) = task_input.get(Self::TERMINATION_WORKFLOW_OUTPUT) {
            if let Object::Map(map) = input {
                output.extend(
                    map.iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect::<HashMap<_, _>>(),
                );
            } else {
                output.insert("output".into(), input.clone());
            }
        }
        output
    }
}
