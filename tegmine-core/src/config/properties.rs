pub struct Properties {
    /// The maximum threshold of the workflow variables payload size in KB beyond which the task
    /// changes will be rejected and the task will be marked as FAILED_WITH_TERMINAL_ERROR.
    /// KILOBYTES
    max_workflow_variables_payload_size_threshold: i32,
    /// The timeout duration to set when a workflow is pushed to the decider queue.
    workflow_offset_timeout_sec: i64,
    ///
    task_pending_time_threshold_sec: i64,
    /// Used to enable/disable asynchronous indexing to elasticsearch.
    async_indexing_enabled: bool,
}

impl Properties {
    pub fn get_max_workflow_variables_payload_size_threshold() -> i32 {
        Properties::default().max_workflow_variables_payload_size_threshold
    }

    pub fn get_workflow_offset_timeout_sec() -> i64 {
        Properties::default().workflow_offset_timeout_sec
    }

    pub fn get_task_pending_time_threshold_sec() -> i64 {
        Properties::default().task_pending_time_threshold_sec
    }

    pub fn is_async_indexing_enabled() -> bool {
        Properties::default().async_indexing_enabled
    }
}

impl Default for Properties {
    fn default() -> Self {
        Self {
            max_workflow_variables_payload_size_threshold: 256,
            workflow_offset_timeout_sec: 30,
            async_indexing_enabled: false,
            task_pending_time_threshold_sec: 60 * 60, // 60min
        }
    }
}
