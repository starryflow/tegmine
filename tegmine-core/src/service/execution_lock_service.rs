use tegmine_common::prelude::*;

pub struct ExecutionLockService;

impl ExecutionLockService {
    pub fn acquire_lock(lock_id: &InlineStr) -> bool {
        true
    }

    pub fn acquire_lock_try_time(lock_id: &InlineStr, time_to_try_ms: i64) -> bool {
        true
    }

    pub fn release_lock(lock_id: &InlineStr) {}

    pub fn delete_lock(lock_id: &InlineStr) {}
}
