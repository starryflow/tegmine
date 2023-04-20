use std::collections::HashMap;

use tegmine_common::prelude::*;
use tegmine_common::TaskType;

use super::workflow_system_task::WorkflowSystemTask;
use crate::config::Properties;
use crate::model::{TaskModel, TaskStatus, WorkflowModel};
use crate::runtime::ExecutionDaoFacade;

pub struct SetVariable;

impl SetVariable {
    fn validate_variables_size(workflow: &WorkflowModel) -> Option<String> {
        let max_threshold = Properties::default().max_workflow_variables_payload_size_threshold;

        let payload_size = Object::estimate_map_memory_used(&workflow.variables);
        if payload_size > max_threshold * 1024 {
            let error_msg = format!("The variables payload size: {} of workflow: {} is greater than the permissible limit: {} bytes", payload_size,workflow.workflow_id,max_threshold);
            error!("{}", error_msg);
            Some(error_msg)
        } else {
            None
        }
    }
}

impl WorkflowSystemTask for SetVariable {
    fn get_task_type(&self) -> &str {
        TaskType::SetVariable.as_ref()
    }

    fn execute(&self, workflow: &mut WorkflowModel, task: &mut TaskModel) -> bool {
        if !task.input_data.is_empty() {
            let mut new_keys = Vec::default();
            let mut previous_values: HashMap<InlineStr, Object> = HashMap::default();

            task.input_data.iter().for_each(|(k, v)| {
                if let Some(value) = workflow.variables.get(k) {
                    previous_values.insert(k.clone(), value.clone());
                } else {
                    new_keys.push(k);
                }
                workflow.variables.insert(k.clone(), v.clone());
                debug!("Task: {} setting value for variable: {}", task.task_id, k);
            });
            if let Some(error_msg) = Self::validate_variables_size(&workflow) {
                task.reason_for_incompletion = error_msg.into();

                // restore previous variables
                previous_values.into_iter().for_each(|(k, v)| {
                    workflow.variables.insert(k, v);
                });
                new_keys.into_iter().for_each(|x| {
                    workflow.variables.remove(x);
                });
                task.status = TaskStatus::Failed;
                return true;
            }
        }

        task.status = TaskStatus::Completed;
        ExecutionDaoFacade::update_workflow(workflow);
        true
    }
}
