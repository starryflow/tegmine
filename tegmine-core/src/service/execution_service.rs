use tegmine_common::prelude::*;

use crate::model::Workflow;
use crate::runtime::ExecutionDaoFacade;
use crate::WorkflowStatus;

pub struct ExecutionService;

impl ExecutionService {
    pub fn get_execution_status(
        workflow_id: &str,
        include_tasks: bool,
    ) -> TegResult<(WorkflowStatus, Option<Workflow>)> {
        if let Some(status) = ExecutionDaoFacade::get_workflow_status(&workflow_id.into()) {
            if status.is_terminal() {
                Ok((
                    status,
                    Some(ExecutionDaoFacade::get_workflow(
                        &workflow_id.into(),
                        include_tasks,
                    )?),
                ))
            } else {
                Ok((status, None))
            }
        } else {
            fmt_err!(NotFound, "can not find workflow: {}", workflow_id)
        }
    }
}
