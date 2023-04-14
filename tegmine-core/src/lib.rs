#![feature(result_option_inspect)]
#![feature(map_try_insert)]

mod config;
mod dao;
mod model;
mod runtime;
mod service;
mod utils;

pub use model::WorkflowStatus;
pub use service::{ExecutionService, MetadataService, TaskService, WorkflowService};

pub fn initialize() {
    utils::V8Utils::set_up_v8_globally();
}

pub fn spawn_event_loop() {
    std::thread::spawn(|| loop {
        runtime::Channel::handle_evaluation_event()
    });

    std::thread::spawn(|| loop {
        runtime::Channel::handle_creation_event()
    });
}

pub fn evaluate_once() -> tegmine_common::prelude::TegResult<()> {
    runtime::Channel::evaluate_once()
}

#[cfg(test)]
mod tests {

    #[test]
    fn test() {
        todo!()
    }
}
