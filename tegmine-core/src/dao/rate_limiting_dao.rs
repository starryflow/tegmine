use tegmine_common::TaskDef;

use crate::model::TaskModel;

/// Rate Limiting implementation
pub struct RateLimitingDao;

impl RateLimitingDao {
    /// Checks if the Task is rate limited or not based on the
    /// `TaskModel::getRateLimitPerFrequency()` and `TaskModel::getRateLimitFrequencyInSeconds()`
    pub fn exceeds_rate_limit_per_frequency(task: &TaskModel, task_def: Option<&TaskDef>) -> bool {
        // TODO
        true
    }
}
