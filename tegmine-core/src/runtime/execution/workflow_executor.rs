use chrono::Utc;
use tegmine_common::prelude::*;

use super::tasks::SystemTaskRegistry;
use super::DeciderService;
use crate::dao::QueueDao;
use crate::model::{TaskModel, TaskStatus, WorkflowModel, WorkflowStatus};
use crate::runtime::dal::ExecutionDaoFacade;
use crate::runtime::event::{WorkflowCreationEvent, WorkflowEvaluationEvent};
use crate::runtime::execution::tasks::Terminate;
use crate::runtime::execution::{terminate_workflow_exception, CREATE_EVENT_CHANNEL};
use crate::runtime::StartWorkflowInput;
use crate::utils::{IdGenerator, QueueUtils};

/// Workflow services provider interface
pub struct WorkflowExecutor;

impl WorkflowExecutor {
    // resetCallbacksForWorkflow

    // rerun

    // restart

    // retry

    // updateAndPushParents

    // retry

    // findLastFailedSubWorkflowIfAny

    // task_to_be_rescheduled

    fn end_execution(
        workflow: &mut WorkflowModel,
        terminate_task: Option<&mut TaskModel>,
    ) -> TegResult<()> {
        if let Some(terminate_task) = terminate_task {
            let termination_status = if let Some(termination_status) = terminate_task
                .workflow_task
                .as_ref()
                .expect("task in termination not none")
                .input_parameters
                .get(&Terminate::get_termination_status_parameter())
            {
                termination_status.as_string()?.as_str()
            } else {
                ""
            };
            let reason = if let Some(reason) = terminate_task
                .workflow_task
                .as_ref()
                .expect("task in termination not none")
                .input_parameters
                .get(&Terminate::get_termination_reason_parameter())
            {
                let reason = reason.as_string()?;
                if reason.trim().is_empty() {
                    format!(
                        "Workflow is {} by TERMINATE task: {}",
                        termination_status, terminate_task.task_id
                    )
                    .into()
                } else {
                    reason.clone()
                }
            } else {
                format!(
                    "Workflow is {} by TERMINATE task: {}",
                    termination_status, terminate_task.task_id
                )
                .into()
            };
            if WorkflowStatus::Failed.as_ref().eq(termination_status) {
                workflow.status = WorkflowStatus::Failed;
                Self::terminate(workflow, workflow.status, Some(terminate_task), reason)?;
            } else {
                workflow.reason_for_incompletion = reason;
                Self::complete_workflow(workflow)?;
            }
        } else {
            Self::complete_workflow(workflow)?;
        };
        Self::cancel_non_terminal_tasks(workflow)?;
        Ok(())
    }

    fn complete_workflow(workflow: &mut WorkflowModel) -> TegResult<()> {
        debug!("Completing workflow execution for {}", workflow.workflow_id);

        if workflow.status == WorkflowStatus::Completed {
            // remove from the sweep queue
            QueueDao::remove(QueueDao::DECIDER_QUEUE, &workflow.workflow_id)?;
            ExecutionDaoFacade::remove_from_pending_workflow(
                &workflow.workflow_definition.name,
                &workflow.workflow_id,
            );
            debug!(
                "Workflow: {} has already been completed.",
                workflow.workflow_id
            );
            return Ok(());
        }

        if workflow.status.is_terminal() {
            return fmt_err!(
                Conflict,
                "Workflow is already in terminal state. Current status: {}",
                workflow.status.as_ref()
            );
        }

        DeciderService::update_workflow_output(workflow, None)?;

        workflow.status = WorkflowStatus::Completed;

        // update the failed reference task names
        let failed_tasks = workflow
            .tasks
            .iter()
            .filter(|x| {
                x.status == TaskStatus::Failed || x.status == TaskStatus::FailedWithTerminalError
            })
            .collect::<Vec<_>>();

        workflow.failed_reference_task_names.extend(
            failed_tasks
                .iter()
                .map(|x| x.reference_task_name.clone())
                .collect::<HashSet<_>>(),
        );

        workflow.failed_task_names.extend(
            failed_tasks
                .iter()
                .map(|x| x.task_def_name.clone())
                .collect::<HashSet<_>>(),
        );

        ExecutionDaoFacade::update_workflow(workflow);
        debug!(
            "Completed workflow execution for {}",
            workflow.workflow_id.clone()
        );
        //  workflowStatusListener.onWorkflowTerminatedIfEnabled(workflow);
        // Monitors.recordWorkflowTermination(

        if workflow.has_parent() {
            Self::update_parent_workflow_task(workflow);
            info!(
                "{} updated parent {} task {}",
                workflow.to_short_string(),
                workflow.parent_workflow_id,
                workflow.parent_workflow_task_id
            );
            Self::expedite_lazy_workflow_evaluation(&workflow.parent_workflow_id);
        }
        // executionLockService.releaseLock(workflow.getWorkflowId());
        // executionLockService.deleteLock(workflow.getWorkflowId());
        Ok(())
    }

    #[allow(unused)]
    pub fn terminate_workflow(workflow: &mut WorkflowModel, reason: InlineStr) -> TegResult<()> {
        let mut workflow = ExecutionDaoFacade::get_workflow_model(&workflow.workflow_id, true)?;
        if workflow.status == WorkflowStatus::Completed {
            str_err!(Conflict, "Cannot terminate a COMPLETED workflow.")
        } else {
            workflow.status = WorkflowStatus::Terminated;
            Self::terminate_workflow_with_failure_workflow(&mut workflow, reason, InlineStr::new())
        }
    }

    pub fn terminate_workflow_with_failure_workflow(
        workflow: &mut WorkflowModel,
        reason: InlineStr,
        failure_workflow: InlineStr,
    ) -> TegResult<()> {
        // executionLockService.acquireLock(workflow.getWorkflowId(), 60000);

        if !workflow.status.is_terminal() {
            workflow.status = WorkflowStatus::Terminated;
        }

        if let Err(e) = DeciderService::update_workflow_output(workflow, None) {
            // catch any failure in this step and continue the execution of terminating workflow
            error!(
                "Failed to update output data for workflow: {}, error: {}",
                workflow.workflow_id, e
            );
            // Monitors.error(CLASS_NAME, "terminateWorkflow");
        }

        // update the failed reference task names
        let failed_tasks = workflow
            .tasks
            .iter()
            .filter(|x| {
                x.status == TaskStatus::Failed || x.status == TaskStatus::FailedWithTerminalError
            })
            .collect::<Vec<_>>();

        workflow.failed_reference_task_names.extend(
            failed_tasks
                .iter()
                .map(|x| x.reference_task_name.clone())
                .collect::<HashSet<_>>(),
        );

        workflow.failed_task_names.extend(
            failed_tasks
                .iter()
                .map(|x| x.task_def_name.clone())
                .collect::<HashSet<_>>(),
        );

        let workflow_id = workflow.workflow_id.clone();
        workflow.reason_for_incompletion = reason.clone();
        ExecutionDaoFacade::update_workflow(workflow);
        //  workflowStatusListener.onWorkflowTerminatedIfEnabled(workflow);
        // Monitors.recordWorkflowTermination(
        info!(
            "Workflow {} is terminated because of {}",
            workflow_id, reason
        );

        let tasks = &workflow.tasks;
        // Remove from the task queue if they were there
        if let Err(e) = tasks.iter().try_for_each(|x| {
            QueueDao::remove(
                QueueUtils::get_queue_name_by_task_model(x).as_str(),
                &x.task_id,
            )
        }) {
            warn!(
                "Error removing task(s) from queue during workflow termination : {}, error: {}",
                workflow_id, e,
            )
        }

        if workflow.has_parent() {
            Self::update_parent_workflow_task(workflow);
            info!(
                "{} updated parent {} task {}",
                workflow.to_short_string(),
                workflow.parent_workflow_id,
                workflow.parent_workflow_task_id
            );
            Self::expedite_lazy_workflow_evaluation(&workflow.parent_workflow_id);
        }

        if !failure_workflow.trim().is_empty() {
            let mut input = HashMap::with_capacity(workflow.input.len());
            input.extend(workflow.input.clone());
            input.insert("workflowId".into(), workflow_id.clone().into());
            input.insert("reason".into(), reason.into());
            input.insert("failureStatus".into(), workflow.status.as_ref().into());
            input.insert(
                "failureTaskId".into(),
                workflow.failed_task_id.as_str().into(),
            );

            let failure_wf_id = IdGenerator::generate();
            let start_workflow_input = StartWorkflowInput::new(
                failure_workflow.clone(),
                input,
                workflow.correlation_id.clone(),
                workflow.task_to_domain.clone(),
                failure_wf_id.clone(),
                workflow_id,
            );

            if let Err(e) = CREATE_EVENT_CHANNEL
                .0
                .send(WorkflowCreationEvent::new(start_workflow_input))
            {
                error!("Failed to start error workflow, error: {}", e);
                workflow.output.insert(
                    "tegmine.failure_workflow".into(),
                    format!(
                        "Error workflow {} failed to start.  reason: {}",
                        failure_workflow,
                        e.to_string()
                    )
                    .into(),
                );
                // Monitors.recordWorkflowStartError(
            }
            workflow
                .output
                .insert("tegmine.failure_workflow".into(), failure_wf_id.into());
            ExecutionDaoFacade::update_workflow(workflow);
        }
        ExecutionDaoFacade::remove_from_pending_workflow(
            &workflow.workflow_definition.name,
            &workflow.workflow_id,
        );

        let result = match Self::cancel_non_terminal_tasks(workflow) {
            Ok(errored_tasks) => {
                if !errored_tasks.is_empty() {
                    fmt_err!(
                        NonTransient,
                        "Error canceling system tasks: {}",
                        errored_tasks.join(",")
                    )
                } else {
                    Ok(())
                }
            }
            Err(e) => Err(e),
        };

        // executionLockService.releaseLock(workflow.getWorkflowId());
        // executionLockService.deleteLock(workflow.getWorkflowId());
        result
    }

    // updateTask

    pub fn handle_workflow_evaluation_event(wee: WorkflowEvaluationEvent) -> TegResult<()> {
        Self::decide(wee.workflow_model)
    }

    /// return true if the workflow has completed (success or failed), false otherwise.
    ///
    /// Note: This method does not acquire the lock on the workflow and should ony be called /
    /// overridden if No locking is required or lock is acquired externally
    pub fn decide(mut workflow: WorkflowModel) -> TegResult<()> {
        if workflow.status.is_terminal() {
            if !workflow.status.is_successful() {
                Self::cancel_non_terminal_tasks(&mut workflow)?;
            }
            return Ok(());
        }

        // we find any sub workflow tasks that have changed
        // and change the workflow/task state accordingly
        // adjustStateIfSubWorkflowChanged(workflow);

        match DeciderService::decide(&mut workflow) {
            Ok(mut outcome) => {
                if outcome.is_complete {
                    Self::end_execution(
                        &mut workflow,
                        outcome.terminate_task.map(|x| from_addr_mut!(x)),
                    )?;
                    return Ok(());
                }

                Self::set_task_domains(&workflow, &mut outcome.tasks_to_be_scheduled);

                let (tasks_to_be_scheduled, tasks_to_be_scheduled_in_outcome) =
                    Self::dedup_and_add_tasks(&mut workflow, outcome.tasks_to_be_scheduled);
                debug!("workflow has {} tasks.", workflow.tasks.len());

                let mut state_changed =
                    Self::schedule_task(&workflow, tasks_to_be_scheduled.as_slice())?; // start

                for task in tasks_to_be_scheduled_in_outcome {
                    let task = from_addr_mut!(task);
                    ExecutionDaoFacade::populate_task_data(task);
                    if SystemTaskRegistry::is_system_task(&task.task_type)
                        && !task.status.is_terminal()
                    {
                        let workflow_system_task = SystemTaskRegistry::get(&task.task_type)?;
                        debug!("find SystemTask: {}", workflow_system_task.get_task_type());
                        if !workflow_system_task.value().as_ref().is_async()
                            && workflow_system_task.execute(&mut workflow, task)
                        {
                            outcome.tasks_to_be_updated.push(task);
                            state_changed = true;
                        }
                    }
                }

                debug!(
                    "find {} tasks to be updated",
                    outcome.tasks_to_be_updated.len()
                );
                if !outcome.tasks_to_be_updated.is_empty() || !tasks_to_be_scheduled.is_empty() {
                    ExecutionDaoFacade::update_tasks(outcome.tasks_to_be_updated.as_slice());
                }

                if state_changed {
                    return Self::decide(workflow);
                }

                if !outcome.tasks_to_be_updated.is_empty() || !tasks_to_be_scheduled.is_empty() {
                    ExecutionDaoFacade::update_workflow(&mut workflow);
                }

                Ok(())
            }
            Err(e) => {
                if e.code() == ErrorCode::TERMINATE_WORKFLOW_CODE {
                    info!(
                        "Execution terminated of workflow: {:?}, error: {}",
                        workflow, e
                    );

                    Self::terminate(
                        &mut workflow,
                        terminate_workflow_exception::STATUS
                            .with(|x| x.take())
                            .unwrap_or(WorkflowStatus::Failed),
                        terminate_workflow_exception::TASK
                            .with(|x| x.take())
                            .take()
                            .as_mut(),
                        e.message().into(),
                    )?;
                    Ok(())
                } else {
                    error!("Error deciding workflow: {:?}, error: {}", workflow, e);
                    Err(e)
                }
            }
        }
    }

    fn cancel_non_terminal_tasks(workflow: &mut WorkflowModel) -> TegResult<Vec<InlineStr>> {
        let mut errored_tasks = Vec::default();

        // Update non-terminal tasks' status to CANCELED
        let workflow_ptr = addr_of_mut!(workflow);
        for task in workflow.tasks.iter_mut() {
            if !task.status.is_terminal() {
                // Cancel the ones which are not completed yet....
                task.status = TaskStatus::Canceled;
                if SystemTaskRegistry::is_system_task(&task.task_type) {
                    let workflow_system_task = SystemTaskRegistry::get(&task.task_type)?;
                    if let Err(e) = workflow_system_task.cancel(from_addr_mut!(workflow_ptr), task)
                    {
                        errored_tasks.push(task.reference_task_name.clone());
                        error!(
                            "Error canceling system task:{}/{} in workflow: {}, error: {}",
                            workflow_system_task.get_task_type(),
                            task.task_id,
                            workflow.workflow_id,
                            e
                        );
                    }
                }
                ExecutionDaoFacade::update_task(task);
            }
        }
        if errored_tasks.is_empty() {
            // workflowStatusListener.onWorkflowFinalizedIfEnabled(workflow);
            if let Err(e) = QueueDao::remove(QueueDao::DECIDER_QUEUE, &workflow.workflow_id) {
                error!(
                    "Error removing workflow: {} from decider queue, error: {}",
                    workflow.workflow_id, e
                );
            }
        }
        Ok(errored_tasks)
    }

    fn dedup_and_add_tasks(
        workflow: &mut WorkflowModel,
        tasks: Vec<TaskModel>,
    ) -> (Vec<*mut TaskModel>, Vec<*mut TaskModel>) {
        let mut deduped_tasks: Vec<*mut TaskModel> = Vec::with_capacity(tasks.len());
        let mut original_tasks: Vec<*mut TaskModel> = Vec::with_capacity(tasks.len());
        for task in tasks {
            if let Some(exist) = workflow
                .tasks
                .iter_mut()
                .find(|x| x.get_task_key().eq(&task.get_task_key()))
            {
                original_tasks.push(exist);
            } else {
                workflow.tasks.push_back(task);
                let recent_push = workflow.tasks.back_mut().expect("not none");
                deduped_tasks.push(recent_push);
                original_tasks.push(recent_push);
            }
        }

        (deduped_tasks, original_tasks)
    }

    pub fn add_task_to_queue(task: &TaskModel) -> TegResult<()> {
        // put in queue
        let task_queue_name = QueueUtils::get_queue_name_by_task_model(task);
        if task.callback_after_seconds > 0 {
            QueueDao::push(
                &task_queue_name,
                &task.task_id,
                task.workflow_priority,
                task.callback_after_seconds,
            );
        } else {
            QueueDao::push(&task_queue_name, &task.task_id, task.workflow_priority, 0);
        }
        debug!(
            "Added task {:?} with priority {} to queue {} with call back seconds {}",
            task, task.workflow_priority, task_queue_name, task.callback_after_seconds
        );
        Ok(())
    }

    fn set_task_domains(workflow: &WorkflowModel, tasks: &mut [TaskModel]) {
        let task_to_domain = &workflow.task_to_domain;
        if !task_to_domain.is_empty() {
            // Step 1: Apply * mapping to all tasks, if present.
            if let Some(domain_str) =
                task_to_domain
                    .get("*")
                    .and_then(|x| if x.trim().is_empty() { None } else { Some(x) })
            {
                let domains = domain_str.split(",").collect::<Vec<_>>();
                tasks.iter_mut().for_each(|x| {
                    // Filter out SystemTask
                    if !SystemTaskRegistry::is_system_task(&x.task_type) {
                        // Check which domain worker is polling
                        // Set the task domain
                        x.domain = Self::get_active_domain(&x.task_type, &domains);
                    }
                });
                // Step 2: Override additional mappings.
                tasks.iter_mut().for_each(|x| {
                    if !SystemTaskRegistry::is_system_task(&x.task_type) {
                        if let Some(task_domain_str) = task_to_domain.get(&x.task_type) {
                            x.domain = Self::get_active_domain(
                                &x.task_type,
                                &task_domain_str.split(",").collect::<Vec<_>>(),
                            );
                        }
                    }
                });
            }
        }
    }

    /// Gets the active domain from the list of domains where the task is to be queued. The domain
    /// list must be ordered. In sequence, check if any worker has polled for last
    /// `activeWorkerLastPollMs`, if so that is the Active domain. When no active domains are found:
    /// If NO_DOMAIN token is provided, return null.
    /// Else, return last domain from list.
    fn get_active_domain(_task_type: &InlineStr, domains: &[&str]) -> InlineStr {
        if domains.is_empty() {
            InlineStr::new()
        } else {
            // domains.iter().filter(|x|!x.eq_ignore_ascii_case("NO_DOMAIN")).
            // map(|x|ExecutionDaoFacade::get_task_poll_data_by_domain(task_type,
            // x.trim())).filter(|x|x.is_some())
            unimplemented!()
        }
    }

    fn schedule_task(workflow: &WorkflowModel, tasks: &[*mut TaskModel]) -> TegResult<bool> {
        let mut started_system_tasks = false;

        if tasks.is_empty() {
            return Ok(false);
        }

        let mut tasks = tasks.iter().map(|&x| from_addr_mut!(x)).collect::<Vec<_>>();

        // Get the highest seq number
        let mut count = workflow.tasks.iter().map(|x| x.seq).max().unwrap_or(0);

        for task in &mut tasks {
            if task.seq == 0 {
                count += 1;
                task.seq = count;
            }
        }

        // metric to track the distribution of number of tasks within a workflow
        // Monitors.recordNumTasksInWorkflow(
        //             workflow.getTasks().size() + tasks.size(),
        //             workflow.getWorkflowName(),
        //             String.valueOf(workflow.getWorkflowVersion()));

        // Save the tasks in the DAO
        ExecutionDaoFacade::create_tasks(tasks.as_mut())?;

        let mut system_task = Vec::default();
        let mut tasks_to_be_queued = Vec::default();
        for task_no_cat in tasks {
            if SystemTaskRegistry::is_system_task(&task_no_cat.task_type) {
                system_task.push(task_no_cat);
            } else {
                tasks_to_be_queued.push(task_no_cat);
            }
        }

        // Traverse through all the system tasks, start the sync tasks, in case of async queue
        // the tasks
        for task in system_task {
            let workflow_system_task = SystemTaskRegistry::get(&task.task_type)?;

            if !task.status.is_terminal() && task.start_time == 0 {
                task.start_time = Utc::now().timestamp_millis();
            }

            if !workflow_system_task.is_async() {
                // start execution of synchronous system tasks
                if let Err(e) = workflow_system_task.start(workflow, &task) {
                    return fmt_err!(
                        NonTransient,
                        "Unable to start system task: {}, {{id: {}, name: {}}}, error: {}",
                        task.task_type,
                        task.task_id,
                        task.task_def_name,
                        e
                    );
                }

                started_system_tasks = true;
                ExecutionDaoFacade::update_task(task);
            } else {
                tasks_to_be_queued.push(task);
            }
        }
        // TODO Exception process

        // On addTaskToQueue failures, ignore the exceptions and let WorkflowRepairService take care
        // of republishing the messages to the queue.
        if let Err(e) = Self::add_tasks_to_queue(&tasks_to_be_queued) {
            let task_ids = tasks_to_be_queued
                .iter()
                .map(|x| x.task_id.clone())
                .collect::<Vec<_>>();
            warn!(
                "Error pushing tasks to the queue: {}, for workflow: {}, error: {}",
                task_ids.join(","),
                workflow.workflow_id,
                e
            );
            //    Monitors.error(CLASS_NAME, "scheduleTask");
        }

        Ok(started_system_tasks)
    }

    fn add_tasks_to_queue(tasks: &[&mut TaskModel]) -> TegResult<()> {
        for task in tasks {
            Self::add_task_to_queue(task)?;
        }
        Ok(())
    }

    fn terminate(
        workflow: &mut WorkflowModel,
        status: WorkflowStatus,
        task: Option<&mut TaskModel>,
        reason: InlineStr,
    ) -> TegResult<()> {
        if !workflow.status.is_terminal() {
            workflow.status = status;
        }

        if let Some(task) = &task {
            if workflow.failed_task_id.is_empty() {
                workflow.failed_task_id = task.task_id.clone();
            }
        }

        let mut failure_workflow = workflow.workflow_definition.failure_workflow.clone();
        if failure_workflow.starts_with("$") {
            // name of the input parameter
            let name = failure_workflow.split(r".").collect::<Vec<_>>()[2];
            failure_workflow = workflow
                .input
                .get(name)
                .and_then(|x| x.as_string().ok())
                .map(|x| x.clone())
                .unwrap_or(InlineStr::new());
        }
        if let Some(task) = task {
            ExecutionDaoFacade::update_task(task);
        }

        Self::terminate_workflow_with_failure_workflow(workflow, reason, failure_workflow)
    }

    fn update_parent_workflow_task(_sub_workflow: &WorkflowModel) {
        unimplemented!()
    }

    /// Pushes workflow id into the decider queue with a higher priority to expedite evaluation.
    fn expedite_lazy_workflow_evaluation(_workflow_id: &InlineStr) {
        unimplemented!()
    }
}
