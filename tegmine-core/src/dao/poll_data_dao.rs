use tegmine_common::prelude::*;

pub struct PollDataDao;

impl PollDataDao {
    /// Updates the `PollData` information with the most recently polled data for a task queue.
    pub fn update_last_poll_data(
        task_def_name: &str,
        domain: &str,
        worker_id: &str,
    ) -> TegResult<()> {
        // TODO
        Ok(())
    }
}
