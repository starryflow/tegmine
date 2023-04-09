use crate::prelude::*;
use crate::WorkflowDef;

pub struct StartWorkflowRequest {
    /// Name of the Workflow. MUST be registered with Tegmine before starting workflow
    pub name: InlineStr,
    /// Workflow version
    pub version: Option<i32>,
    /// JSON object with key value params, that can be used by downstream tasks
    pub input: HashMap<InlineStr, Object>,

    /// Unique Id that correlates multiple Workflow executions
    pub correlation_id: InlineStr,
    /// Task domains helps support task development. The idea is same "task definition" can be
    /// implemented in different "domains". A domain is some arbitrary name that the developer
    /// controls. So when the workflow is started, the caller can specify, out of all the tasks in
    /// the workflow, which tasks need to run in a specific domain, this domain is then used to
    /// poll for task on the client side to execute it.
    pub task_to_domain: HashMap<InlineStr, InlineStr>,
    /// An adhoc Workflow Definition to run, without registering.
    pub workflow_def: Option<WorkflowDef>,
    /// This is taken care of by Java client.
    pub external_input_payload_storage_path: InlineStr,
    /// Priority level for the tasks within this workflow execution. Possible values are between 0
    /// - 99.
    pub priority: i32,
}

impl StartWorkflowRequest {
    pub fn new(name: InlineStr, version: Option<i32>, input: HashMap<InlineStr, Object>) -> Self {
        Self {
            name,
            version,
            input,
            correlation_id: InlineStr::default(),
            task_to_domain: HashMap::default(),
            workflow_def: None,
            external_input_payload_storage_path: InlineStr::default(),
            priority: 0,
        }
    }
}

impl TryFrom<serde_json::Value> for StartWorkflowRequest {
    type Error = ErrorCode;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let task_to_domain = if let Some(json) = value.get("taskToDomain") {
            if json.as_null().is_none() {
                let mut task_to_domain = HashMap::default();
                for (k, v) in json
                    .as_object()
                    .ok_or(ErrorCode::IllegalArgument("taskToDomain invalid"))?
                {
                    if let Some(v) = v.as_str() {
                        task_to_domain.insert(k.into(), v.into());
                    } else {
                        return fmt_err!(
                            IllegalArgument,
                            "taskToDomain invalid, key/value must be string"
                        );
                    }
                }
                task_to_domain
            } else {
                HashMap::default()
            }
        } else {
            HashMap::default()
        };

        let workflow_def = if let Some(workflow_def) = value.get("workflowDef") {
            Some(WorkflowDef::try_from(workflow_def)?)
        } else {
            None
        };

        Ok(Self {
            name: value
                .get("name")
                .and_then(|x| x.as_str())
                .ok_or(ErrorCode::IllegalArgument("name not found"))?
                .trim()
                .into(),
            version: value
                .get("version")
                .and_then(|x| x.as_i64())
                .map(|x| x as i32),
            input: Object::convert_jsonmap_to_hashmap(
                value
                    .get("input")
                    .and_then(|x| x.as_object())
                    .ok_or(ErrorCode::IllegalArgument("input invalid"))?,
            ),
            correlation_id: value
                .get("correlationId")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .trim()
                .into(),
            task_to_domain,
            workflow_def,
            external_input_payload_storage_path: value
                .get("externalInputPayloadStoragePath")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .trim()
                .into(),
            priority: value
                .get("priority")
                .unwrap_or(&serde_json::json!(0))
                .as_i64()
                .map(|x| x as i32)
                .and_then(|x| if x < 0 || x > 99 { None } else { Some(x) })
                .ok_or(ErrorCode::IllegalArgument(
                    "priority must in range [0..=99]",
                ))?,
        })
    }
}
