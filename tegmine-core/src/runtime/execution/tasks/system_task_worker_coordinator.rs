use dashmap::DashMap;
use once_cell::sync::Lazy;
use tegmine_common::prelude::*;

use crate::runtime::execution::tasks::system_task_worker::SystemTaskWorker;
use crate::WorkflowSystemTask;

static ASYNC_SYSTEM_TASKS: Lazy<DashMap<InlineStr, Arc<Box<dyn WorkflowSystemTask>>>> =
    Lazy::new(|| DashMap::new());

pub struct SystemTaskWorkerCoordinator;

impl SystemTaskWorkerCoordinator {
    pub fn init_system_task_executor() {
        ASYNC_SYSTEM_TASKS
            .iter()
            .for_each(move |task| SystemTaskWorker::start_polling(Arc::clone(task.value())));

        info!(
            "{} initialized with {} async tasks",
            "system_task_worker_coordinator",
            ASYNC_SYSTEM_TASKS.len()
        );
    }

    pub fn register_async_system_task(system_task: Box<dyn WorkflowSystemTask>) -> TegResult<()> {
        if !system_task.is_async() {
            return Err(ErrorCode::IllegalArgument(
                "The registered WorkflowSystemTask must be an asyn task",
            ));
        }
        ASYNC_SYSTEM_TASKS.insert(InlineStr::from(system_task.get_task_type()), Arc::from(system_task));
        Ok(())
    }
}
