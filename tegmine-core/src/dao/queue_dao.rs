use std::time::Duration;

use chrono::Utc;
use dashmap::DashMap;
use keyed_priority_queue::KeyedPriorityQueue;
use rhai::Instant;
use tegmine_common::prelude::*;

/// DAO responsible for managing queuing for the tasks.
pub struct QueueDao;

static QUEUES_PRIORITY: Lazy<DashMap<InlineStr, KeyedPriorityQueue<InlineStr, i64>>> =
    Lazy::new(|| DashMap::new());
static QUEUES_MESSAGE: Lazy<DashMap<InlineStr, DashMap<InlineStr, Message>>> =
    Lazy::new(|| DashMap::new());

impl QueueDao {
    pub const DECIDER_QUEUE: &'static str = "_deciderQueue";

    pub fn push(queue_name: &str, id: &InlineStr, priority: i32, offset_time_in_second: i64) {
        let mut message = Message::new(id.clone());
        message.timeout_millis = offset_time_in_second * 1000;
        if priority >= 0 && priority <= 99 {
            message.priority = priority;
        }
        if !QUEUES_PRIORITY.contains_key(queue_name) {
            QUEUES_PRIORITY.insert(queue_name.into(), KeyedPriorityQueue::new());
            QUEUES_MESSAGE.insert(queue_name.into(), DashMap::new());
        }

        let score = Utc::now().timestamp_millis() * 100
            + message.timeout_millis * 100
            + message.priority as i64;

        QUEUES_PRIORITY
            .get_mut(queue_name)
            .expect("not empty")
            .value_mut()
            .push(message.id.clone(), -score);
        QUEUES_MESSAGE
            .get_mut(queue_name)
            .expect("not empty")
            .value_mut()
            .insert(message.id.clone(), message);
    }

    /// If the popped messages are not acknowledge in a timely manner, they are pushed back
    /// into the queue.
    ///
    /// return list of elements from the named queue
    pub fn pop(queue_name: &str, count: i32, timeout_ms: i32) -> TegResult<Vec<InlineStr>> {
        if let Some(mut queue) = QUEUES_PRIORITY.get_mut(queue_name) {
            let mut wait_count = count;
            let mut wait_time = timeout_ms;
            let mut message_ids = Vec::with_capacity(count as usize);

            let mut start = Instant::now();
            loop {
                if wait_count <= 0 {
                    break;
                }

                let mut found = false;
                if let Some(message_pri) = queue.value().peek() {
                    if (-message_pri.1 / 100) <= Utc::now().timestamp_millis() {
                        let message_id = message_pri.0.clone();

                        queue.value_mut().remove(&message_id);
                        message_ids.push(message_id);
                        found = true;
                        wait_count -= 1;
                    }
                }

                if !found {
                    wait_time -= start.elapsed().as_millis() as i32;
                    start = Instant::now();

                    if wait_time > 0 {
                        // at least sleep 10ms
                        std::thread::sleep(Duration::from_millis((wait_time as u64).min(10)));
                    } else {
                        break;
                    }
                }
            }

            Ok(message_ids)
        } else {
            Ok(Vec::default())
        }
    }

    pub fn remove(queue_name: &str, message_id: &InlineStr) -> TegResult<()> {
        if let Some(mut queue) = QUEUES_PRIORITY.get_mut(queue_name) {
            queue.value_mut().remove(message_id);
        }

        if let Some(mut queue) = QUEUES_MESSAGE.get_mut(queue_name) {
            queue.value_mut().remove(message_id);
        }

        Ok(())
    }

    /// return true if the message was found and ack'ed
    pub fn ack(_queue_name: &str, _message_id: &InlineStr) -> bool {
        // no need to implement ack
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

struct Message {
    id: InlineStr,
    timeout_millis: i64,
    /// 0-99, 0 is highest priority
    priority: i32,
}

impl Message {
    fn new(id: InlineStr) -> Self {
        Self {
            id,
            timeout_millis: 0,
            priority: 0,
        }
    }
}
