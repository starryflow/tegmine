use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};
use dashmap::DashMap;
use futures::executor::{ThreadPool, ThreadPoolBuilder};
use tegmine_common::prelude::*;
use tegmine_common::StartWorkflowRequest;

use super::WorkflowExecutor;
use crate::runtime::event::{WorkflowCreationEvent, WorkflowEvaluationEvent};
use crate::runtime::StartWorkflowOperation;
use crate::ExecutionService;
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
        .expect("thread pool create failed")
});

pub static WAITING_QUEUE: Lazy<DashMap<InlineStr, Sender<()>>> = Lazy::new(|| DashMap::new());
pub static ASYNC_WAITING_QUEUE: Lazy<DashMap<InlineStr, tokio::sync::oneshot::Sender<()>>> =
    Lazy::new(|| DashMap::new());

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

                // notify caller if using async_execute
                if let Some((_, sender)) = ASYNC_WAITING_QUEUE.remove(&id) {
                    if let Err(_) = sender.send(()) {
                        error!(
                            "failed to send to caller after workflow finished, workflow id: {}",
                            id,
                        );
                    }
                    return;
                }

                // notify caller if using block_execute
                if let Some((_, sender)) = WAITING_QUEUE.remove(&id) {
                    if let Err(e) = sender.send(()) {
                        error!(
                            "failed to send to caller after workflow finished, workflow id: {}, {}",
                            id, e
                        );
                    }
                    return;
                }
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
        timeout: Duration,
    ) -> TegResult<HashMap<InlineStr, Object>> {
        let (sender, receiver) = crossbeam_channel::bounded(0);

        let workflow_instance_id = WorkflowService::start_workflow(request)?;
        WAITING_QUEUE.insert(workflow_instance_id.clone(), sender);

        receiver.recv_timeout(timeout).map_err(|e| {
            warn!(
                "failed to execute workflow, workflow id: {}, {}",
                workflow_instance_id, e
            );
            ErrorCode::ExecutionException(format!(
                "failed to execute workflow, it tasks too long, workflow id: {}",
                workflow_instance_id
            ))
        })?;
        Self::fetch_execute_output(workflow_instance_id.as_str())
    }

    pub async fn async_execute(
        request: StartWorkflowRequest,
    ) -> TegResult<HashMap<InlineStr, Object>> {
        let (sender, receiver) = tokio::sync::oneshot::channel();

        let workflow_instance_id = WorkflowService::start_workflow(request)?;
        ASYNC_WAITING_QUEUE.insert(workflow_instance_id.clone(), sender);

        receiver.await.map_err(|e| {
            warn!(
                "failed to execute workflow, workflow id: {}, {}",
                workflow_instance_id, e
            );
            ErrorCode::ExecutionException(format!(
                "failed to execute workflow, workflow id: {}",
                workflow_instance_id
            ))
        })?;
        Self::fetch_execute_output(workflow_instance_id.as_str())
    }

    fn fetch_execute_output(workflow_instance_id: &str) -> TegResult<HashMap<InlineStr, Object>> {
        let (workflow_status, workflow) =
            ExecutionService::get_execution_status(workflow_instance_id, false)?;

        if workflow_status.is_terminal() && workflow_status.is_successful() {
            let output = workflow
                .ok_or(ErrorCode::ExecutionException(format!(
                    "failed to get workflow output, workflow id: {}",
                    workflow_instance_id
                )))?
                .workflow
                .output;
            return Ok(output);
        } else {
            return Err(ErrorCode::ExecutionException(format!(
                "workflow execute failed, workflow id: {}",
                workflow_instance_id
            )));
        }
    }
}
