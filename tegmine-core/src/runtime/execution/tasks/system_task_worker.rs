use std::thread;
use std::time::Duration;

use futures::executor::{ThreadPool, ThreadPoolBuilder};
use tegmine_common::prelude::*;

use crate::dao::QueueDao;
use crate::metrics::Monitors;
use crate::runtime::execution::AsyncSystemTaskExecutor;
use crate::utils::{QueueUtils, SemaphoreUtil};
use crate::{ExecutionService, WorkflowSystemTask};

const POLL_INTERVAL: u64 = 50;

const THREAD_COUNT: i32 = 10;
static SEMAPHORE_UTIL: Lazy<SemaphoreUtil> = Lazy::new(|| SemaphoreUtil::new(THREAD_COUNT));
static POOL: Lazy<ThreadPool> = Lazy::new(|| {
    ThreadPoolBuilder::new()
        .pool_size(THREAD_COUNT as usize)
        .create()
        .expect("thread pool create failed")
});

pub struct SystemTaskWorker;
impl SystemTaskWorker {
    pub fn start_polling(system_task: Arc<Box<dyn WorkflowSystemTask>>) {
        let task_type = InlineStr::from(system_task.get_task_type());
        Self::start_polling_with_queue_name(system_task, &task_type);
    }

    pub fn start_polling_with_queue_name(
        system_task: Arc<Box<dyn WorkflowSystemTask>>,
        queue_name: &InlineStr,
    ) {
        let queue_name = queue_name.clone();
        thread::spawn(move || {
            const DELAY: u64 = 1000;
            thread::sleep(Duration::from_millis(DELAY));
            loop {
                if let Err(e) = Self::poll_and_execute(Arc::clone(&system_task), &queue_name) {
                    error!("failed to poll_and_execute, {}", e);
                }
                thread::sleep(Duration::from_millis(POLL_INTERVAL));
            }
        });
    }

    pub fn poll_and_execute(
        system_task: Arc<Box<dyn WorkflowSystemTask>>,
        queue_name: &InlineStr,
    ) -> TegResult<()> {
        fn _poll_and_execute(
            system_task: Arc<Box<dyn WorkflowSystemTask>>,
            queue_name: &str,
            messages_to_acquire: i32,
        ) -> TegResult<()> {
            if messages_to_acquire <= 0 || !SEMAPHORE_UTIL.acquire_slots(messages_to_acquire) {
                // no available slots, do not poll
                Monitors::record_system_task_worker_polling_limited(queue_name);
                return Ok(());
            }

            trace!(
                "Polling queue: {} with {} slots acquired",
                queue_name, messages_to_acquire
            );

            let polled_task_ids = QueueDao::pop(queue_name, messages_to_acquire, 200)?;

            Monitors::record_task_poll(queue_name);
            trace!(
                "Polling queue:{}, got {} tasks",
                queue_name,
                polled_task_ids.len()
            );

            if polled_task_ids.len() > 0 {
                // Immediately release unused slots when number of messages acquired is less than
                // acquired slots
                if polled_task_ids.len() < messages_to_acquire as usize {
                    SEMAPHORE_UTIL
                        .complete_processing(messages_to_acquire - polled_task_ids.len() as i32);
                }

                for task_id in polled_task_ids {
                    if !task_id.is_empty() {
                        debug!(
                            "Task: {} from queue: {} being sent to the workflow executor",
                            task_id, queue_name
                        );
                        Monitors::record_task_poll_count_no_domain(queue_name, 1);
                        ExecutionService::ack_task_received_by_task_id(&task_id);

                        let system_task_arc = Arc::clone(&system_task);
                        POOL.spawn_ok(async {
                            if let Err(e) = AsyncSystemTaskExecutor::execute(
                                system_task_arc,
                                &InlineStr::from(task_id),
                            ) {
                                error!("AsyncSystemTaskExecutor execute failed, {}", e)
                            }
                            SEMAPHORE_UTIL.complete_processing(1);
                        });
                    } else {
                        SEMAPHORE_UTIL.complete_processing(1);
                    }
                }
            } else {
                // no task polled, release permit
                SEMAPHORE_UTIL.complete_processing(messages_to_acquire);
            }

            Ok(())
        }

        let task_name = QueueUtils::get_task_type(queue_name);

        let messages_to_acquire = SEMAPHORE_UTIL.available_slots();

        if let Err(e) = _poll_and_execute(system_task, queue_name, messages_to_acquire) {
            // release the permit if exception is thrown during polling, because the thread would
            // not be busy
            SEMAPHORE_UTIL.complete_processing(messages_to_acquire);
            Monitors::record_task_poll_error_no_domain(&task_name, "SystemTaskWorker");
            error!("Error polling system task in queue:{} {}", queue_name, e);
        }
        Ok(())
    }
}
