use chrono::Utc;
use numtoa::NumToA;
use strum_macros::{AsRefStr, EnumString};
use tegmine_common::prelude::*;
use tegmine_common::{TaskDef, WorkflowTask};

use super::Task;

#[derive(Clone, Debug)]
pub struct TaskModel {
    pub task_type: InlineStr,
    pub status: TaskStatus,
    pub reference_task_name: InlineStr,
    pub retry_count: i32,
    pub seq: i32,
    pub correlation_id: InlineStr,
    pub poll_count: i32,
    pub task_def_name: InlineStr,
    /// Time when the task was scheduled
    pub scheduled_time: i64,
    /// Time when the task was first polled
    pub start_time: i64,
    /// Time when the task completed executing
    pub end_time: i64,
    /// Time when the task was last updated
    pub update_time: i64,
    pub start_delay_in_seconds: i32,
    pub retried_task_id: InlineStr,
    pub retried: bool,
    pub executed: bool,
    pub callback_from_worker: bool,
    pub response_timeout_seconds: i64,
    pub workflow_instance_id: InlineStr,
    pub workflow_type: InlineStr,
    pub task_id: InlineStr,
    pub reason_for_incompletion: InlineStr,
    pub callback_after_seconds: i64,
    pub worker_id: InlineStr,
    pub workflow_task: Option<WorkflowTask>,
    pub domain: InlineStr,
    pub input_message: Object,
    pub output_message: Object,
    pub rate_limit_per_frequency: i32,
    pub rate_limit_frequency_in_seconds: i32,
    pub external_input_payload_storage_path: InlineStr,
    pub external_output_payload_storage_path: InlineStr,
    pub workflow_priority: i32,
    pub execution_name_space: InlineStr,
    pub isolation_group_id: InlineStr,
    pub iteration: i32,
    pub sub_workflow_id: InlineStr,
    /// Timeout after which the wait task should be marked as completed
    pub wait_timeout: i64,
    /// Used to note that a sub workflow associated with SUB_WORKFLOW task has an action performed
    /// on it directly.
    pub sub_workflow_changed: bool,
    pub input_payload: HashMap<InlineStr, Object>,
    pub output_payload: HashMap<InlineStr, Object>,
    pub input_data: HashMap<InlineStr, Object>,
    pub output_data: HashMap<InlineStr, Object>,
}

impl TaskModel {
    pub fn new(status: TaskStatus) -> Self {
        Self {
            task_type: InlineStr::new(),
            status,
            reference_task_name: InlineStr::new(),
            retry_count: 0,
            seq: 0,
            correlation_id: InlineStr::new(),
            poll_count: 0,
            task_def_name: InlineStr::new(),
            scheduled_time: 0,
            start_time: 0,
            end_time: 0,
            update_time: 0,
            start_delay_in_seconds: 0,
            retried_task_id: InlineStr::new(),
            retried: false,
            executed: false,
            callback_from_worker: true,
            response_timeout_seconds: 0,
            workflow_instance_id: InlineStr::new(),
            workflow_type: InlineStr::new(),
            task_id: InlineStr::new(),
            reason_for_incompletion: InlineStr::new(),
            callback_after_seconds: 0,
            worker_id: InlineStr::new(),
            workflow_task: None,
            domain: InlineStr::new(),
            input_message: Object::Null,
            output_message: Object::Null,
            rate_limit_per_frequency: 0,
            rate_limit_frequency_in_seconds: 0,
            external_input_payload_storage_path: InlineStr::new(),
            external_output_payload_storage_path: InlineStr::new(),
            workflow_priority: 0,
            execution_name_space: InlineStr::new(),
            isolation_group_id: InlineStr::new(),
            iteration: 0,
            sub_workflow_id: InlineStr::new(),
            wait_timeout: 0,
            sub_workflow_changed: false,
            input_payload: HashMap::new(),
            output_payload: HashMap::new(),
            input_data: HashMap::new(),
            output_data: HashMap::new(),
        }
    }

    pub fn get_task_definition(&self) -> Option<&TaskDef> {
        self.workflow_task
            .as_ref()
            .and_then(|x| x.task_definition.as_ref())
    }

    pub fn get_task_key(&self) -> InlineStr {
        let mut task_name = self.reference_task_name.clone();
        task_name.push_str("_");
        task_name.push_str(self.retry_count.numtoa_str(10, &mut [0u8; 16]));
        task_name
    }

    pub fn get_queue_wait_time(&self) -> i64 {
        if self.start_time > 0 && self.scheduled_time > 0 {
            if self.update_time > 0 && self.callback_after_seconds > 0 {
                let wait_time = Utc::now().timestamp_millis()
                    - (self.update_time + self.callback_after_seconds * 1000);
                if wait_time > 0 {
                    wait_time
                } else {
                    0
                }
            } else {
                self.start_time - self.scheduled_time
            }
        } else {
            0
        }
    }

    pub fn to_task(self) -> Task {
        Task { inner: self }
    }
}

#[derive(Clone, Copy, Debug, EnumString, AsRefStr, PartialEq, Eq)]
pub enum TaskStatus {
    InProgress,
    Canceled,
    Failed,
    FailedWithTerminalError,
    Completed,
    CompletedWithErrors,
    Scheduled,
    TimedOut,
    Skipped,
}

impl TaskStatus {
    pub fn is_terminal(&self) -> bool {
        match self {
            TaskStatus::InProgress | TaskStatus::Scheduled => false,
            _ => true,
        }
    }

    pub fn is_successful(&self) -> bool {
        match self {
            TaskStatus::Canceled
            | TaskStatus::Failed
            | TaskStatus::FailedWithTerminalError
            | TaskStatus::TimedOut => false,
            _ => true,
        }
    }

    pub fn is_retriable(&self) -> bool {
        match self {
            TaskStatus::Canceled | TaskStatus::FailedWithTerminalError | TaskStatus::Skipped => {
                false
            }
            _ => true,
        }
    }
}
