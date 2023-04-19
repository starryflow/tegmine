use std::collections::VecDeque;
use std::str::FromStr;

use strum_macros::{AsRefStr, EnumString};

use crate::prelude::*;
use crate::{TaskType, WorkflowTask};

#[derive(Clone, Debug)]
pub struct WorkflowDef {
    /// Name of the workflow
    pub name: InlineStr,
    /// Description of the workflow
    pub description: InlineStr,
    /// Numeric field used to identify the version of the schema. Use incrementing numbers.
    pub version: i32,
    /// An array of task configurations.
    pub tasks: Vec<WorkflowTask>,
    /// List of input parameters. Used for documenting the required inputs to workflow
    pub input_parameters: Vec<InlineStr>,
    /// JSON template used to generate the output of the workflow
    pub output_parameters: HashMap<InlineStr, Object>,
    /// Default input values.
    pub input_template: HashMap<InlineStr, Object>,
    /// Workflow to be run on current Workflow failure. Useful for cleanup or post actions on
    /// failure.
    pub failure_workflow: InlineStr,
    /// Current Tegmine Schema version. schemaVersion 1 is discontinued.
    pub schema_version: i32,
    /// Flag to allow Workflow restarts
    pub restartable: bool,
    /// Enable status callback.
    pub workflow_status_listener_enabled: bool,
    /// Email address of the team that owns the workflow
    pub owner_email: InlineStr,
    /// The timeout in seconds after which the workflow will be marked as TIMED_OUT if it hasn't
    /// been moved to a terminal state
    pub timeout_seconds: i32,
    /// Workflow's timeout policy
    pub timeout_policy: TimeoutPolicy,
    pub variables: HashMap<InlineStr, Object>,

    pub create_time: i64,
    pub update_time: i64,
}

impl WorkflowDef {
    pub fn get_next_task(&self, task_reference_name: &str) -> Option<&WorkflowTask> {
        if let Some(workflow_task) = self.get_task_by_ref_name(task_reference_name) {
            if workflow_task.type_.eq(TaskType::Terminate.as_ref()) {
                return None;
            }
        }

        let mut iterator = self.tasks.iter();
        while let Some(task) = iterator.next() {
            if task.task_reference_name.eq(task_reference_name) {
                // If taskReferenceName matches, break out
                break;
            }
            if let Some(next_task) = task.next(task_reference_name, None) {
                return Some(next_task);
            } else if task.type_.eq(TaskType::DoWhile.as_ref())
                && !task.task_reference_name.eq(task_reference_name)
                && task.has(task_reference_name)
            {
                // If the task is child of Loop Task and at last position, return null.
                return None;
            }

            if task.has(task_reference_name) {
                break;
            }
        }

        if let Some(next) = iterator.next() {
            Some(next)
        } else {
            None
        }
    }

    pub fn get_task_by_ref_name(&self, task_reference_name: &str) -> Option<&WorkflowTask> {
        self.collect_tasks()
            .into_iter()
            .filter(|&x| x.task_reference_name.eq(task_reference_name))
            .collect::<VecDeque<_>>()
            .pop_front()
    }

    pub fn collect_tasks(&self) -> Vec<&WorkflowTask> {
        let mut tasks = Vec::default();
        for workflow_task in &self.tasks {
            tasks.extend(workflow_task.collect_tasks())
        }
        debug!("collect {} tasks for {:?}", tasks.len(), self);
        tasks
    }

    pub fn populate_tasks(&mut self, populate_fn: fn(&mut WorkflowTask)) {
        for workflow_task in &mut self.tasks {
            workflow_task.populate_tasks(populate_fn);
        }
    }
}

impl TryFrom<&serde_json::Value> for WorkflowDef {
    type Error = ErrorCode;
    fn try_from(value: &serde_json::Value) -> Result<Self, ErrorCode> {
        // Optional
        let input_parameters: Vec<InlineStr> = if value.get("inputParameters").is_none() {
            Vec::default()
        } else {
            let mut input_parameters: Vec<InlineStr> = Vec::default();
            for input_param in value
                .get("inputParameters")
                .and_then(|x| x.as_array())
                .ok_or_else(|| ErrorCode::IllegalArgument("inputParameters invalid, not a array"))?
            {
                if let Some(input_p) = input_param.as_str() {
                    input_parameters.push(input_p.trim().into());
                } else {
                    return str_err!(
                        IllegalArgument,
                        "inputParameters invalid, not a string in array"
                    );
                }
            }
            input_parameters
        };

        // Optional
        let output_parameters: HashMap<InlineStr, Object> =
            if value.get("outputParameters").is_none() {
                HashMap::default()
            } else {
                Object::convert_jsonmap_to_hashmap(
                    value
                        .get("outputParameters")
                        .and_then(|x| x.as_object())
                        .ok_or_else(|| ErrorCode::IllegalArgument("outputParameters invalid"))?,
                )
            };

        // Optional
        let input_template: HashMap<InlineStr, Object> = if value.get("inputTemplate").is_none() {
            HashMap::default()
        } else {
            Object::convert_jsonmap_to_hashmap(
                value
                    .get("inputTemplate")
                    .and_then(|x| x.as_object())
                    .ok_or_else(|| ErrorCode::IllegalArgument("inputTemplate invalid"))?,
            )
        };

        Ok(Self {
            name: value
                .get("name")
                .and_then(|x| x.as_str())
                .ok_or_else(|| ErrorCode::IllegalArgument("WorkflowDef: name not found"))?
                .trim()
                .into(),
            description: value
                .get("description")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .trim()
                .into(),
            version: value
                .get("version")
                .unwrap_or(&serde_json::json!(0))
                .as_i64()
                .ok_or_else(|| ErrorCode::IllegalArgument("WorkflowDef: version invalid"))?
                as i32,
            tasks: WorkflowTask::try_from_jsonlist(
                value
                    .get("tasks")
                    .and_then(|x| x.as_array())
                    .ok_or_else(|| {
                        ErrorCode::IllegalArgument("WorkflowDef: tasks not found or not array")
                    })?,
            )?,
            input_parameters,
            output_parameters,
            input_template,
            failure_workflow: value
                .get("failureWorkflow")
                .unwrap_or(&serde_json::json!(""))
                .as_str()
                .ok_or_else(|| ErrorCode::IllegalArgument("WorkflowDef: failureWorkflow invalid"))?
                .trim()
                .into(),
            schema_version: 2,
            restartable: value
                .get("restartable")
                .unwrap_or(&serde_json::json!(true))
                .as_bool()
                .ok_or_else(|| ErrorCode::IllegalArgument("WorkflowDef: restartable invalid"))?,
            workflow_status_listener_enabled: value
                .get("workflowStatusListenerEnabled")
                .unwrap_or(&serde_json::json!(false))
                .as_bool()
                .ok_or_else(|| {
                    ErrorCode::IllegalArgument("WorkflowDef: workflowStatusListenerEnabled invalid")
                })?,
            owner_email: value
                .get("ownerEmail")
                .unwrap_or(&serde_json::json!(""))
                .as_str()
                .ok_or_else(|| ErrorCode::IllegalArgument("WorkflowDef: ownerEmail invalid"))?
                .trim()
                .into(),
            timeout_seconds: value
                .get("timeoutSeconds")
                .unwrap_or(&serde_json::json!(0))
                .as_i64()
                .ok_or_else(|| {
                    ErrorCode::IllegalArgument("WorkflowDef: timeoutSeconds not found")
                })? as i32,
            timeout_policy: TimeoutPolicy::from_str(
                value
                    .get("timeoutPolicy")
                    .unwrap_or(&serde_json::json!("TIME_OUT_WF"))
                    .as_str()
                    .ok_or_else(|| {
                        ErrorCode::IllegalArgument("WorkflowDef: timeoutPolicy invalid")
                    })?
                    .trim(),
            )
            .map_err(|_| ErrorCode::IllegalArgument("WorkflowDef: timeoutPolicy invalid"))?,
            variables: HashMap::default(),
            create_time: 0,
            update_time: 0,
        })
    }
}

#[derive(Clone, Copy, Debug, AsRefStr, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeoutPolicy {
    /// Workflow is marked as TIMED_OUT and terminated
    TimeOutWf,
    /// Registers a counter (workflow_failure with status tag set to TIMED_OUT)
    AlertOnly,
}
