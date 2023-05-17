use either::Either;
use fancy_regex::Regex;
use smartstring::SmartString;
use tegmine_common::prelude::*;
use tegmine_common::{EnvUtils, TaskDef, TaskUtils, WorkflowDef};

use crate::model::WorkflowModel;

/// Used to parse and resolve the JSONPath bindings in the workflow and task definitions.
pub struct ParametersUtils;

impl ParametersUtils {
    pub fn get_task_input(
        input_params: &HashMap<InlineStr, Object>,
        workflow: &WorkflowModel,
        task_definition: Option<&TaskDef>,
        task_id: Option<&InlineStr>,
    ) -> TegResult<HashMap<InlineStr, Object>> {
        let mut input_params = if !input_params.is_empty() {
            input_params.clone()
        } else {
            HashMap::default()
        };
        if let Some(task_definition) = task_definition {
            task_definition.input_template.iter().for_each(|(k, v)| {
                let _ = input_params.try_insert(k.clone(), v.clone());
            });
        }

        let workflow_params: HashMap<InlineStr, Object> = HashMap::from([
            (SmartString::from("input"), workflow.input.clone().into()),
            (SmartString::from("output"), workflow.output.clone().into()),
            (SmartString::from("status"), workflow.status.as_ref().into()),
            (
                SmartString::from("workflowId"),
                workflow.workflow_id.clone().into(),
            ),
            (
                SmartString::from("parentWorkflowId"),
                workflow.parent_workflow_id.clone().into(),
            ),
            (
                SmartString::from("parentWorkflowTaskId"),
                workflow.parent_workflow_task_id.clone().into(),
            ),
            (
                SmartString::from("workflowType"),
                workflow.workflow_definition.name.clone().into(),
            ),
            (
                SmartString::from("version"),
                workflow.workflow_definition.version.clone().into(),
            ),
            (
                SmartString::from("correlationId"),
                workflow.correlation_id.clone().into(),
            ),
            (
                SmartString::from("reasonForIncompletion"),
                workflow.reason_for_incompletion.clone().into(),
            ),
            (
                SmartString::from("schemaVersion"),
                workflow.workflow_definition.schema_version.into(),
            ),
            (
                SmartString::from("variables"),
                workflow.variables.clone().into(),
            ),
        ]);

        let mut input_map: HashMap<InlineStr, Object> =
            HashMap::from([(InlineStr::from("workflow"), workflow_params.into())]);
        // For new workflow being started the list of tasks will be empty
        if let ControlFlow::Break(e) = workflow
            .tasks
            .iter()
            .map(|x| &x.reference_task_name)
            .try_for_each(|x| match workflow.get_task_by_ref_name(&x) {
                Ok(Some(x)) => {
                    let mut task_params: HashMap<InlineStr, Object> = HashMap::default();
                    task_params.insert("input".into(), x.input_data.clone().into());
                    task_params.insert("output".into(), x.output_data.clone().into());
                    task_params.insert("taskType".into(), x.task_type.clone().into());
                    task_params.insert("status".into(), x.status.as_ref().into());
                    task_params.insert(
                        "referenceTaskName".into(),
                        x.reference_task_name.clone().into(),
                    );
                    task_params.insert("retryCount".into(), x.retry_count.into());
                    task_params.insert("correlationId".into(), x.correlation_id.clone().into());
                    task_params.insert("pollCount".into(), x.poll_count.into());
                    task_params.insert("taskDefName".into(), x.task_def_name.clone().into());
                    task_params.insert("scheduledTime".into(), x.scheduled_time.into());
                    task_params.insert("startTime".into(), x.start_time.into());
                    task_params.insert("endTime".into(), x.end_time.into());
                    task_params.insert(
                        "workflowInstanceId".into(),
                        x.workflow_instance_id.clone().into(),
                    );
                    task_params.insert("taskId".into(), x.task_id.clone().into());
                    task_params.insert(
                        "reasonForIncompletion".into(),
                        x.reason_for_incompletion.clone().into(),
                    );
                    task_params.insert(
                        "callbackAfterSeconds".into(),
                        x.callback_after_seconds.into(),
                    );
                    task_params.insert("workerId".into(), x.worker_id.clone().into());
                    task_params.insert("iteration".into(), x.iteration.into());

                    let input_key = if x.iteration > 0 {
                        TaskUtils::remove_iteration_from_task_ref_name(&x.reference_task_name)
                            .into()
                    } else {
                        x.reference_task_name.clone()
                    };
                    input_map.insert(input_key, task_params.into());
                    ControlFlow::Continue(())
                }
                Ok(None) => ControlFlow::Continue(()),
                Err(e) => ControlFlow::Break(e),
            })
        {
            return Err(e);
        }

        let mut document_context = Either::Left(input_map);
        let mut replaced_task_input = Self::replace(input_params, &mut document_context, task_id);
        if let Some(task_definition) = task_definition {
            if !task_definition.input_template.is_empty() {
                // If input for a given key resolves to null, try replacing it with one from
                // inputTemplate, if it exists.
                for (k, v) in replaced_task_input.iter_mut() {
                    if v.is_null() {
                        let value = task_definition
                            .input_template
                            .get(k)
                            .map(|x| x.clone())
                            .unwrap_or(Object::Null);
                        let _ = std::mem::replace(v, value);
                    }
                }
            }
        }
        Ok(replaced_task_input)
    }

    fn replace(
        input: HashMap<InlineStr, Object>,
        document_context: &mut Either<HashMap<InlineStr, Object>, serde_json::Value>,
        task_id: Option<&InlineStr>,
    ) -> HashMap<InlineStr, Object> {
        let mut replace_map = HashMap::with_capacity(input.len());
        for (k, v) in input {
            let new_value = match v {
                Object::String(value) => {
                    Self::replace_variables(value, document_context, task_id).into()
                }
                Object::Map(value) => Self::replace(value, document_context, task_id).into(),
                Object::List(value) => Self::replace_list(value, document_context, task_id).into(),
                v @ _ => v,
            };
            replace_map.insert(k, new_value);
        }
        replace_map
    }

    fn replace_list(
        input_list: Vec<Object>,
        document_context: &mut Either<HashMap<InlineStr, Object>, serde_json::Value>,
        task_id: Option<&InlineStr>,
    ) -> Vec<Object> {
        let mut replace_list = Vec::with_capacity(input_list.len());
        for v in input_list {
            let new_value = match v {
                Object::String(value) => {
                    Self::replace_variables(value, document_context, task_id).into()
                }
                Object::Map(value) => Self::replace(value, document_context, task_id).into(),
                Object::List(value) => Self::replace_list(value, document_context, task_id).into(),
                v @ _ => v,
            };
            replace_list.push(new_value);
        }
        replace_list
    }

    fn replace_variables(
        param_string: InlineStr,
        document_context: &mut Either<HashMap<InlineStr, Object>, serde_json::Value>,
        task_id: Option<&InlineStr>,
    ) -> Object {
        lazy_static! {
            static ref DOLLAR_REGEX: Regex =
                Regex::new(r"(?=(?<!\$)\$\{)|(?<=})").expect("regex compile error");
            static ref DOUBLE_DOLLAR_REGEX: Regex =
                Regex::new(r"\$\$\{").expect("regex compile error");
        }

        let mut values = Vec::default();
        let mut matches = DOLLAR_REGEX.find_iter(&param_string);
        let mut last = 0;
        let text = matches.text();
        loop {
            match matches.next() {
                None => {
                    if last >= text.len() {
                        break;
                    } else {
                        let s = &text[last..];
                        debug!("find matched: {}", s);
                        values.push(s);

                        last = text.len() + 1; // Next call will return None
                    }
                }
                Some(Ok(m)) => {
                    if last != m.start() {
                        let matched = &text[last..m.start()];
                        debug!("find matched: {}", matched);
                        values.push(matched);
                    }
                    last = m.end();
                }
                Some(Err(e)) => {
                    error!("regex match failed, error: {}", e);
                }
            }
        }
        let mut converted_values: Vec<Object> = Vec::with_capacity(values.len());
        for v in values {
            if v.starts_with("${") && v.ends_with("}") {
                let param_path = &v[2..v.len() - 1];
                // if the paramPath is blank, meaning no value in between ${ and }
                // like ${}, ${  } etc, set the value to empty string
                if param_path.trim().is_empty() {
                    converted_values.push(InlineStr::from("").into());
                    continue;
                }
                if let Some(sys_value) = EnvUtils::get_system_parameters_value(param_path, task_id)
                {
                    converted_values.push(sys_value.into());
                } else {
                    converted_values.push(Object::read(document_context, param_path))
                }
            } else if v.contains("$${") {
                converted_values
                    .push(InlineStr::from(DOUBLE_DOLLAR_REGEX.replace(v, r"\${")).into());
            } else {
                converted_values.push(v.into());
            }
        }

        let ret_obj = converted_values[0].clone();
        // If the parameter String was "v1 v2 v3" then make sure to stitch it
        if converted_values.len() > 1 {
            let mut ret_obj = converted_values[0].to_string();
            for (i, val) in converted_values.into_iter().enumerate() {
                if i == 0 {
                    ret_obj = val.to_string();
                } else {
                    ret_obj.push_str(&val.to_string());
                }
            }
            return ret_obj.into();
        }

        ret_obj
    }

    pub fn get_workflow_input(
        workflow_def: &WorkflowDef,
        input_params: &mut HashMap<InlineStr, Object>,
    ) {
        workflow_def.input_template.iter().for_each(|(k, v)| {
            let _ = input_params.try_insert(k.clone(), v.clone());
        })
    }
}
