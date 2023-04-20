use tegmine_common::prelude::*;

/// Interface implemented by a distributed lock client.
pub struct Lock;

impl Lock {
    pub fn acquire_lock_try_and_lease_time(
        lock_id: &InlineStr,
        time_to_try_ms: i64,
        lease_time_ms: i64,
    ) -> bool {
        unimplemented!()
    }

    pub fn release_lock(lock_id: &InlineStr) {
        unimplemented!()
    }

    pub fn delete_lock(lock_id: &InlineStr) {
        unimplemented!()
    }
}
