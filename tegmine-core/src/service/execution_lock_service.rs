use tegmine_common::prelude::*;

use crate::config::Properties;
use crate::metrics::Monitors;
use crate::runtime::Lock;

pub struct ExecutionLockService;

impl ExecutionLockService {
    ///  Tries to acquire lock with reasonable time_to_try duration and lease time. Exits if a lock
    /// cannot be acquired. Considering that the workflow decide can be triggered through multiple
    /// entry points, and periodically through the sweeper service, do not block on acquiring the
    /// lock, as the order of execution of decides on a workflow doesn't matter.
    pub fn acquire_lock(lock_id: &InlineStr) -> bool {
        Self::acquire_lock_try_and_lease_time(
            lock_id,
            Properties::default().lock_time_to_try_ms,
            Properties::default().lock_lease_time_ms,
        )
    }

    pub fn acquire_lock_try_time(lock_id: &InlineStr, time_to_try_ms: i64) -> bool {
        Self::acquire_lock_try_and_lease_time(
            lock_id,
            time_to_try_ms,
            Properties::default().lock_lease_time_ms,
        )
    }

    pub fn acquire_lock_try_and_lease_time(
        lock_id: &InlineStr,
        time_to_try_ms: i64,
        lease_time_ms: i64,
    ) -> bool {
        if Properties::default().workflow_execution_lock_enabled {
            if !Lock::acquire_lock_try_and_lease_time(lock_id, time_to_try_ms, lease_time_ms) {
                debug!(
                    "Thread {:?} failed to acquire lock to lockId {}.",
                    std::thread::current().id(),
                    lock_id
                );
                Monitors::record_acquire_lock_unsuccessful();
                return false;
            }
            debug!(
                "Thread {:?} acquired lock to lockId {}.",
                std::thread::current().id(),
                lock_id
            );
        }
        true
    }

    pub fn release_lock(lock_id: &InlineStr) {
        if Properties::default().workflow_execution_lock_enabled {
            Lock::release_lock(lock_id);
            debug!(
                "Thread {:?} released lock to lockId {}.",
                std::thread::current().id(),
                lock_id
            );
        }
    }

    pub fn delete_lock(lock_id: &InlineStr) {
        if Properties::default().workflow_execution_lock_enabled {
            Lock::delete_lock(lock_id);
            debug!(
                "Thread {:?} deleted lockId {}.",
                std::thread::current().id(),
                lock_id
            );
        }
    }
}
