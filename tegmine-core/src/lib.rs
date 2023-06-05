#![feature(result_option_inspect)]
#![feature(map_try_insert)]

mod config;
mod dao;
mod metrics;
mod model;
mod runtime;
mod service;
mod utils;

use crossbeam_channel::Sender;
pub use model::{TaskModel, TaskStatus, WorkflowModel, WorkflowStatus};
pub use runtime::{
    SystemTaskRegistry, TaskMapper, TaskMapperContext, TaskMapperRegistry, WorkflowSystemTask,
};
pub use service::{ExecutionService, MetadataService, TaskService, WorkflowService};
use tegmine_common::prelude::InlineStr;
pub use utils::ParametersUtils;

pub fn initialize() {
    // utils::V8Utils::set_up_v8_globally();
}

pub fn spawn_event_loop() {
    std::thread::spawn(|| loop {
        runtime::Channel::handle_evaluation_event_paralle()
    });

    std::thread::spawn(|| loop {
        runtime::Channel::handle_creation_event()
    });
}

pub fn evaluate_once() -> tegmine_common::prelude::TegResult<()> {
    runtime::Channel::evaluate_once()
}

pub fn block_execute_workflow(
    start_workflow_request: tegmine_common::StartWorkflowRequest,
    sender: Sender<()>,
) -> tegmine_common::prelude::TegResult<InlineStr> {
    runtime::Channel::block_execute(start_workflow_request, sender)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test() {
        todo!()
    }
}
