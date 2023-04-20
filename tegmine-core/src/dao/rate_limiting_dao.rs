use tegmine_common::TaskDef;

use crate::model::TaskModel;

/// Rate Limiting implementation
pub struct RateLimitingDao;

impl RateLimitingDao {
    /// Checks if the Task is rate limited or not based on the
    /// `TaskModel::rate_limit_per_frequency` and `TaskModel::rate_limit_frequency_in_seconds`
    ///
    /// return true: If the `TaskModel` is rateLimited false: If the `TaskModel` is not rateLimited
    pub fn exceeds_rate_limit_per_frequency(task: &TaskModel, task_def: Option<&TaskDef>) -> bool {
        // TODO
        false
    }
}
