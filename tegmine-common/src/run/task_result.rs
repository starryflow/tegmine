use super::task_exec_log::TaskExecLog;
use crate::prelude::*;

/// Result of the task execution.
#[derive(Debug)]
pub struct TaskResult {
    pub workflow_instance_id: InlineStr,
    pub task_id: InlineStr,
    pub reason_for_incompletion: InlineStr,
    pub callback_after_seconds: i64,
    pub worker_id: InlineStr,
    pub status: TaskResultStatus,
    pub output_data: HashMap<InlineStr, Object>,
    pub output_message: Object,
    pub logs: Vec<TaskExecLog>,
    pub external_output_payload_storage_path: InlineStr,
    pub sub_workflow_id: InlineStr,
    pub extend_lease: bool,
}

impl TaskResult {}

#[derive(Debug)]
pub enum TaskResultStatus {
    InProgress,
    Failed,
    FailedWithTerminalError,
    Completed,
}
