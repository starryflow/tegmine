use tegmine_common::prelude::*;

use crate::model::Workflow;
use crate::runtime::ExecutionDaoFacade;

pub struct ExecutionService;

impl ExecutionService {
    pub fn get_execution_status(workflow_id: &str, include_tasks: bool) -> TegResult<Workflow> {
        ExecutionDaoFacade::get_workflow(&workflow_id.into(), include_tasks)
    }
}
