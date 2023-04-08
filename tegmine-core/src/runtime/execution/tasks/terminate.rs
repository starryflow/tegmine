use tegmine_common::prelude::*;
use tegmine_common::TaskType;

use super::workflow_system_task::WorkflowSystemTask;

pub struct Terminate;

impl WorkflowSystemTask for Terminate {
    fn get_task_type(&self) -> &str {
        TaskType::Terminate.as_ref()
    }
}

impl Terminate {
    const TERMINATION_STATUS_PARAMETER: &'static str = "terminationStatus";
    const TERMINATION_REASON_PARAMETER: &'static str = "terminationReason";

    pub fn get_termination_status_parameter() -> InlineStr {
        Self::TERMINATION_STATUS_PARAMETER.into()
    }
    pub fn get_termination_reason_parameter() -> InlineStr {
        Self::TERMINATION_REASON_PARAMETER.into()
    }
}
