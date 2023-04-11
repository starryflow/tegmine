use strum_macros::*;

use super::WorkflowModel;

pub struct Workflow {
    pub workflow: WorkflowModel,
}

impl Workflow {
    pub fn new(workflow: WorkflowModel) -> Self {
        Self { workflow }
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
