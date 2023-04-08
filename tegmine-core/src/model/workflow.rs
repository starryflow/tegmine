use strum_macros::*;

use super::WorkflowModel;

pub struct Workflow {
    pub inner: WorkflowModel,
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
