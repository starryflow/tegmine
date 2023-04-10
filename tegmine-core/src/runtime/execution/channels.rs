use crossbeam_channel::{Receiver, Sender};
use tegmine_common::prelude::*;

use super::WorkflowExecutor;
use crate::runtime::event::{WorkflowCreationEvent, WorkflowEvaluationEvent};
use crate::runtime::StartWorkflowOperation;

pub static CREATE_EVENT_CHANNEL: Lazy<(
    Sender<WorkflowCreationEvent>,
    Receiver<WorkflowCreationEvent>,
)> = Lazy::new(|| crossbeam_channel::unbounded());

pub static EVAL_EVENT_CHANNEL: Lazy<(
    Sender<WorkflowEvaluationEvent>,
    Receiver<WorkflowEvaluationEvent>,
)> = Lazy::new(|| crossbeam_channel::unbounded());

pub struct Channel;

impl Channel {
    pub fn handle_creation_event() {
        if let Ok(wce) = CREATE_EVENT_CHANNEL.1.recv() {
            let _ = StartWorkflowOperation::handle_workflow_creation_event(wce);
        }
    }

    pub fn handle_evaluation_event() {
        if let Ok(wee) = EVAL_EVENT_CHANNEL.1.recv() {
            let _ = WorkflowExecutor::handle_workflow_evaluation_event(wee);
        }
    }

    pub fn evaluate_once() -> TegResult<()> {
        let wee = EVAL_EVENT_CHANNEL
            .1
            .try_recv()
            .map_err(|_| ErrorCode::NotFound("Evaluation Event not found"))?;
        WorkflowExecutor::handle_workflow_evaluation_event(wee)?;
        Ok(())
    }
}
