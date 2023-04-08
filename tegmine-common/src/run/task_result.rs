/// Result of the task execution.
pub struct TaskResult {}

impl TaskResult {}

pub enum TaskResultStatus {
    InProgress,
    Failed,
    FailedWithTerminalError,
    Completed,
}
