pub struct Properties {
    /// The timeout duration to set when a workflow is pushed to the decider queue.
    pub workflow_offset_timeout_sec: i64,
    /// The time (in seconds) for which a task execution will be postponed if being rate limited or
    /// concurrent execution limited.
    pub task_execution_postpone_duration_sec: i64,
    /// Used to enable/disable asynchronous indexing to elasticsearch.
    pub async_indexing_enabled: bool,

    /// Used to enable/disable the workflow execution lock.
    pub workflow_execution_lock_enabled: bool,
    /// The time (in milliseconds) for which the lock is leased for.
    pub lock_lease_time_ms: i64,
    /// The time (in milliseconds) for which the thread will block in an attempt to acquire the
    /// lock.
    pub lock_time_to_try_ms: i64,

    /// The maximum threshold of the workflow variables payload size in KB beyond which the task
    /// changes will be rejected and the task will be marked as FAILED_WITH_TERMINAL_ERROR.
    /// KILOBYTES
    pub max_workflow_variables_payload_size_threshold: i32,
    ///
    pub task_pending_time_threshold_sec: i64,
}

impl Default for Properties {
    fn default() -> Self {
        Self {
            workflow_offset_timeout_sec: 30,
            task_execution_postpone_duration_sec: 60,
            async_indexing_enabled: false,
            workflow_execution_lock_enabled: false,
            lock_lease_time_ms: 60000,
            lock_time_to_try_ms: 500,
            max_workflow_variables_payload_size_threshold: 256,
            task_pending_time_threshold_sec: 60 * 60, // 60min
        }
    }
}
