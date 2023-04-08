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

pub fn example() {
    std::thread::spawn(|| runtime::Channel::handle_evaluation_event());

    std::thread::spawn(|| runtime::Channel::handle_creation_event());
}

#[cfg(test)]
mod tests {

    #[test]
    fn test() {
        todo!()
    }
}
