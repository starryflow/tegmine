use strum_macros::*;

use super::{TaskModel, WorkflowModel};

pub struct Workflow {
    pub status: WorkflowStatus,
    pub workflow: Option<WorkflowModel>,
    pub tasks: Vec<TaskModel>,
}

impl Workflow {
    pub fn new(
        status: WorkflowStatus,
        workflow: Option<WorkflowModel>,
        tasks: Vec<TaskModel>,
    ) -> Self {
        Self {
            status,
            workflow,
            tasks,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, AsRefStr)]
pub enum WorkflowStatus {
    Running,
    Completed,
    Failed,
    TimedOut,
    Terminated,
    Paused,
}

impl WorkflowStatus {
    pub fn is_terminal(&self) -> bool {
        match self {
            WorkflowStatus::Running | WorkflowStatus::Paused => false,
            _ => true,
        }
    }

    pub fn is_successful(&self) -> bool {
        match self {
            WorkflowStatus::Completed | WorkflowStatus::Paused => true,
            _ => false,
        }
    }
}
