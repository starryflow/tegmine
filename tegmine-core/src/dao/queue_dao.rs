use tegmine_common::prelude::*;

pub struct QueueDao;

impl QueueDao {
    pub const DECIDER_QUEUE: &'static str = "_deciderQueue";

    pub fn push(_queue_name: &str, _id: &InlineStr, _priority: i32, _offset_time_in_second: i64) {}

    pub fn remove(_queue_name: &str, _id: &InlineStr) -> TegResult<()> {
        Ok(())
    }
}
