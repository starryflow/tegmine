use std::str::FromStr;

use strum_macros::{AsRefStr, EnumString};

use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct TaskDef {
    /// Task Name. Unique name of the Task that resonates with its function.
    pub name: InlineStr,
    /// Description of the task
    pub description: InlineStr,
    /// Number of retries to attempt when a Task is marked as failure
    /// Defaults to 3 with maximum allowed capped at 10
    pub retry_count: i32,
    /// Mechanism for the retries
    pub retry_logic: RetryLogic,
    /// Time to wait before retries
    /// Defaults to 60 seconds
    pub retry_delay_seconds: i32,
    /// Task's timeout policy
    /// Defaults to TIME_OUT_WF
    pub timeout_policy: TimeoutPolicy,
    /// Time in seconds, after which the task is marked as TIMED_OUT if not completed after
    /// transitioning to InProgress status for the first time
    /// No timeouts if set to 0
    pub timeout_seconds: i32,
    /// If greater than 0, the task is rescheduled if not updated with a status after this time
    /// (heartbeat mechanism). Useful when the worker polls for the task but fails to complete due
    /// to errors/network failure.
    /// Defaults to 3600
    pub response_timeout_seconds: i32,
    /// Time in seconds, after which the task is marked as TIMED_OUT if not polled by a worker
    /// No timeouts if set to 0
    pub poll_timeout_seconds: i32,
    /// Array of keys of task's expected input. Used for documenting task's input.
    pub input_keys: Vec<InlineStr>,
    /// Array of keys of task's expected output. Used for documenting task's output.
    pub output_keys: Vec<InlineStr>,
    /// Define default input values.
    pub input_template: HashMap<InlineStr, Object>,
    /// Number of tasks that can be executed at any given time
    pub concurrent_exec_limit: Option<i32>,
    /// Sets the rate limit frequency window.
    pub rate_limit_frequency_in_seconds: Option<i32>,
    /// Sets the max number of tasks that can be given to workers within window.
    pub rate_limit_per_frequency: Option<i32>,
    /// Email address of the team that owns the task
    pub owner_email: InlineStr,

    pub isolation_group_id: InlineStr,
    pub execution_name_space: InlineStr,
    /// Backoff scale factor. Applicable for LINEAR_BACKOFF
    pub backoff_scale_factor: i32,

    pub created_by: InlineStr,
    pub create_time: i64,
    pub updated_by: InlineStr,
    pub update_time: i64,
}

impl TaskDef {
    pub const ONE_HOUR_SECS: i32 = 3600;

    pub fn concurrency_limit(&self) -> i32 {
        self.concurrent_exec_limit.unwrap_or(0)
    }

    pub fn get_response_timeout_seconds(&self) -> i32 {
        if self.response_timeout_seconds == 0 {
            if self.timeout_seconds == 0 {
                Self::ONE_HOUR_SECS
            } else {
                self.timeout_seconds
            }
        } else {
            self.response_timeout_seconds
        }
    }
}

impl TaskDef {
    pub fn new(name: &str) -> Self {
        Self {
            name: InlineStr::from(name),
            description: InlineStr::new(),
            retry_count: 3,
            retry_logic: RetryLogic::Fixed,
            retry_delay_seconds: 60,
            timeout_policy: TimeoutPolicy::TimeOutWf,
            timeout_seconds: 0,
            response_timeout_seconds: Self::ONE_HOUR_SECS,
            poll_timeout_seconds: 0,
            input_keys: Vec::default(),
            output_keys: Vec::default(),
            input_template: HashMap::default(),
            concurrent_exec_limit: None,
            rate_limit_frequency_in_seconds: None,
            rate_limit_per_frequency: None,
            owner_email: InlineStr::new(),
            isolation_group_id: InlineStr::new(),
            execution_name_space: InlineStr::new(),
            backoff_scale_factor: 1,
            created_by: InlineStr::new(),
            create_time: 0,
            updated_by: InlineStr::new(),
            update_time: 0,
        }
    }
}

impl TryFrom<&serde_json::Value> for TaskDef {
    type Error = ErrorCode;
    fn try_from(value: &serde_json::Value) -> Result<Self, Self::Error> {
        let input_keys = if let Some(json) = value.get("inputKeys") {
            if json.as_null().is_none() {
                let mut input_keys = Vec::default();
                for input_key in json
                    .as_array()
                    .ok_or(ErrorCode::IllegalArgument("inputKeys invalid"))?
                {
                    if let Some(input_k) = input_key.as_str() {
                        input_keys.push(input_k.trim().into());
                    } else {
                        return str_err!(IllegalArgument, "inputKeys invalid");
                    }
                }
                input_keys
            } else {
                Vec::default()
            }
        } else {
            Vec::default()
        };

        let output_keys = if let Some(json) = value.get("outputKeys") {
            if json.as_null().is_none() {
                let mut output_keys = Vec::default();
                for output_key in json
                    .as_array()
                    .ok_or(ErrorCode::IllegalArgument("outputKeys invalid"))?
                {
                    if let Some(out_k) = output_key.as_str() {
                        output_keys.push(out_k.trim().into());
                    } else {
                        return str_err!(IllegalArgument, "outputKeys invalid");
                    }
                }
                output_keys
            } else {
                Vec::default()
            }
        } else {
            Vec::default()
        };

        let input_template = if value.get("inputTemplate").is_none() {
            HashMap::default()
        } else {
            Object::convert_jsonmap_to_hashmap(
                value
                    .get("inputTemplate")
                    .and_then(|x| x.as_object())
                    .ok_or(ErrorCode::IllegalArgument("inputTemplate invalid"))?,
            )
        };

        Ok(Self {
            name: value
                .get("name")
                .and_then(|x| x.as_str())
                .ok_or(ErrorCode::IllegalArgument("name not found"))?
                .trim()
                .into(),
            description: value
                .get("description")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .trim()
                .into(),
            retry_count: value
                .get("retryCount")
                .unwrap_or(&serde_json::json!(3))
                .as_i64()
                .map(|x| x as i32)
                .and_then(|x| if x < 0 || x > 10 { None } else { Some(x) })
                .ok_or(ErrorCode::IllegalArgument(
                    "retryCount must in range [0..=10]",
                ))?,
            retry_logic: RetryLogic::from_str(
                value
                    .get("retryLogic")
                    .and_then(|x| x.as_str())
                    .ok_or(ErrorCode::IllegalArgument("retryLogic not found"))?
                    .trim()
                    .into(),
            )
            .map_err(|_| ErrorCode::IllegalArgument("retryLogic invalid"))?,
            retry_delay_seconds: value
                .get("retryDelaySeconds")
                .unwrap_or(&serde_json::json!(60))
                .as_i64()
                .ok_or(ErrorCode::IllegalArgument("retryDelaySeconds invalid"))?
                as i32,
            timeout_policy: TimeoutPolicy::from_str(
                value
                    .get("timeoutPolicy")
                    .and_then(|x| x.as_str())
                    .unwrap_or("TIME_OUT_WF")
                    .trim()
                    .into(),
            )
            .map_err(|_| ErrorCode::IllegalArgument("timeoutPolicy invalid"))?,
            timeout_seconds: value
                .get("timeoutSeconds")
                .unwrap_or(&serde_json::json!(0))
                .as_i64()
                .ok_or(ErrorCode::IllegalArgument("timeoutSeconds invalid"))?
                as i32,
            response_timeout_seconds: value
                .get("responseTimeoutSeconds")
                .unwrap_or(&serde_json::json!(3600))
                .as_i64()
                .ok_or(ErrorCode::IllegalArgument("responseTimeoutSeconds invalid"))?
                as i32,
            poll_timeout_seconds: value
                .get("pollTimeoutSeconds")
                .unwrap_or(&serde_json::json!(0))
                .as_i64()
                .and_then(|x| if (x as i32) < 0 { None } else { Some(x as i32) })
                .ok_or(ErrorCode::IllegalArgument("pollTimeoutSeconds invalid"))?,
            input_keys,
            output_keys,
            input_template,
            concurrent_exec_limit: value
                .get("concurrentExecLimit")
                .and_then(|x| x.as_i64())
                .map(|x| x as i32),
            rate_limit_frequency_in_seconds: value
                .get("rateLimitFrequencyInSeconds")
                .and_then(|x| x.as_i64())
                .map(|x| x as i32),
            rate_limit_per_frequency: value
                .get("rateLimitPerFrequency")
                .and_then(|x| x.as_i64())
                .map(|x| x as i32),
            owner_email: value
                .get("ownerEmail")
                .unwrap_or(&serde_json::json!(""))
                .as_str()
                .ok_or(ErrorCode::IllegalArgument("ownerEmail invalid"))?
                .trim()
                .into(),
            isolation_group_id: InlineStr::default(),
            execution_name_space: InlineStr::default(),
            backoff_scale_factor: 1,
            created_by: InlineStr::default(),
            create_time: 0,
            updated_by: InlineStr::default(),
            update_time: 0,
        })
    }
}

#[derive(Clone, Copy, Debug, AsRefStr, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeoutPolicy {
    /// Retries the task again
    Retry,
    /// Workflow is marked as TIMED_OUT and terminated. This is the default value.
    TimeOutWf,
    /// Registers a counter (task_timeout)
    AlertOnly,
}

#[derive(Clone, Copy, Debug, AsRefStr, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum RetryLogic {
    /// Reschedule the task after retry_delay_seconds
    Fixed,
    /// Reschedule the task after retry_delay_seconds * (2 ^ attempt_number)
    ExponentialBackoff,
    /// Reschedule after retry_delay_seconds * backoff_rate * attempt_number
    LinearBackoff,
}
