use tegmine_common::prelude::*;

pub struct QueueDao;

impl QueueDao {
    pub const DECIDER_QUEUE: &'static str = "_deciderQueue";

    pub fn push(_queue_name: &str, _id: &InlineStr, _priority: i32, _offset_time_in_second: i64) {}

    /// return list of elements from the named queue
    pub fn pop(_queue_name: &str, count: i32, timeout: i32) -> TegResult<Vec<InlineStr>> {
        Ok(Vec::default())
    }

    pub fn remove(_queue_name: &str, _id: &InlineStr) -> TegResult<()> {
        Ok(())
    }

    /// return true if the message was found and ack'ed
    pub fn ack(_queue_name: &str, message_id: &InlineStr) -> bool {
        // TODO
        true
    }

    /// Postpone a given message with postponeDurationInSeconds, so that the message won't be
    /// available for further polls until specified duration. By default, the message is removed and
    /// pushed backed with postponeDurationInSeconds to be backwards compatible.
    pub fn postpone(
        queue_name: &str,
        message_id: &InlineStr,
        priority: i32,
        postpone_duration_in_seconds: i64,
    ) -> TegResult<bool> {
        Self::remove(queue_name, message_id)?;
        Self::push(
            queue_name,
            message_id,
            priority,
            postpone_duration_in_seconds,
        );
        Ok(true)
    }
}
