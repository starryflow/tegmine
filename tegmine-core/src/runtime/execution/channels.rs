use crossbeam_channel::{Receiver, Sender};
use dashmap::{DashMap, DashSet};
use futures::executor::{ThreadPool, ThreadPoolBuilder};
use tegmine_common::prelude::*;
use tegmine_common::StartWorkflowRequest;

use super::WorkflowExecutor;
use crate::runtime::event::{WorkflowCreationEvent, WorkflowEvaluationEvent};
use crate::runtime::StartWorkflowOperation;
use crate::WorkflowService;
pub static CREATE_EVENT_CHANNEL: Lazy<(
    Sender<WorkflowCreationEvent>,
    Receiver<WorkflowCreationEvent>,
)> = Lazy::new(|| crossbeam_channel::unbounded());

pub static EVAL_EVENT_CHANNEL: Lazy<(
    Sender<WorkflowEvaluationEvent>,
    Receiver<WorkflowEvaluationEvent>,
)> = Lazy::new(|| crossbeam_channel::unbounded());

const THREAD_POOL_SIZE: usize = 20;
static POOL: Lazy<ThreadPool> = Lazy::new(|| {
    ThreadPoolBuilder::new()
        .pool_size(THREAD_POOL_SIZE)
        .create()
        .unwrap()
});

pub static WAITING_QUEUE: Lazy<DashMap<InlineStr, Sender<()>>> = Lazy::new(|| DashMap::new());

pub struct Channel;

impl Channel {
    pub fn handle_creation_event() {
        if let Ok(wce) = CREATE_EVENT_CHANNEL.1.recv() {
            let _ = StartWorkflowOperation::handle_workflow_creation_event(wce);
        }
    }

    pub fn handle_evaluation_event_paralle() {
        if let Ok(wee) = EVAL_EVENT_CHANNEL.1.recv() {
            POOL.spawn_ok(async {
                let id = wee.workflow_model.workflow_id.clone();
                let _ = WorkflowExecutor::handle_workflow_evaluation_event(wee);

                // notify caller if using block_execute
                if let Some(sender) = WAITING_QUEUE.get(&id) {
                    sender.value().send(());
                }
                WAITING_QUEUE.remove(&id);
            });
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

    pub fn block_execute(
        request: StartWorkflowRequest,
        sender: Sender<()>,
    ) -> TegResult<InlineStr> {
        let workflow_instance_id = WorkflowService::start_workflow(request)?;
        WAITING_QUEUE.insert(workflow_instance_id.clone(), sender);
        Ok(workflow_instance_id)
    }
}
