use std::collections::VecDeque;

use chrono::Utc;
use tegmine_common::prelude::*;
use tegmine_common::{
    RetryLogic, TaskDef, TaskTimeoutPolicy, TaskType, TaskUtils, TimeoutPolicy, WorkflowTask,
};

use super::tasks::SystemTaskRegistry;
use crate::config::Properties;
use crate::dao::MetadataDao;
use crate::model::{TaskModel, TaskStatus, WorkflowModel, WorkflowStatus};
use crate::runtime::execution::mapper::{TaskMapperContext, TaskMapperRegistry};
use crate::runtime::execution::terminate_workflow_exception;
use crate::utils::{IdGenerator, ParametersUtils};

/// Decider evaluates the state of the workflow by inspecting the current state along with the
/// blueprint. The result of the evaluation is either to schedule further tasks, complete/fail the
/// workflow or do nothing.
pub struct DeciderService;

impl DeciderService {
    pub fn decide(workflow: &mut WorkflowModel) -> TegResult<DeciderOutcome> {
        // In case of a new workflow the list of tasks will be empty.
        let tasks = &workflow.tasks;
        // Filter the list of tasks and include only tasks that are not executed,
        // not marked to be skipped and not ready for rerun.
        // For a new workflow, the list of unprocessed_tasks will be empty
        let unprocessed_tasks = tasks
            .iter()
            .filter(|&x| x.status != TaskStatus::Skipped && x.executed)
            .collect::<Vec<_>>();
        debug!("find {} unprocessed tasks.", unprocessed_tasks.len());

        let mut tasks_to_be_scheduled = Vec::default();
        if unprocessed_tasks.is_empty() {
            // this is the flow that the new workflow will go through
            tasks_to_be_scheduled = Self::start_workflow(workflow)?;
        }
        Self::decide_(workflow, tasks_to_be_scheduled)
    }

    fn decide_(
        workflow: &mut WorkflowModel,
        pre_scheduled_tasks: Vec<TaskModel>,
    ) -> TegResult<DeciderOutcome> {
        let mut out_come = DeciderOutcome::new();

        if workflow.status.is_terminal() {
            // you cannot evaluate a terminal workflow
            debug!(
                "Workflow {:?} is already finished. Reason: {}",
                workflow, workflow.reason_for_incompletion
            );
            return Ok(out_come);
        }

        Self::check_workflow_timeout(workflow)?;

        if workflow.status == WorkflowStatus::Paused {
            debug!("Workflow {} is paused", workflow.workflow_id);
            return Ok(out_come);
        }

        let mut pending_tasks = Vec::default();
        let mut executed_task_ref_names = HashSet::new();
        let mut has_successful_terminate_task = false;
        for task in &workflow.tasks {
            // Filter the list of tasks and include only tasks that are not retried, not executed
            // marked to be skipped and not part of System tasks that is DECISION, FORK, JOIN
            // This list will be empty for a new workflow being started
            if !task.retried && task.status != TaskStatus::Skipped && !task.executed {
                pending_tasks.push(task as *const TaskModel as *mut TaskModel);
            }

            // Get all the tasks that have not completed their lifecycle yet
            // This list will be empty for a new workflow
            if task.executed {
                executed_task_ref_names.insert(task.reference_task_name.clone());
            }

            if TaskType::Terminate.as_ref().eq(task.task_type.as_str())
                && task.status.is_terminal()
                && task.status.is_successful()
            {
                has_successful_terminate_task = true;
                out_come.terminate_task = Some(task as *const TaskModel as *mut TaskModel);
            }
        }

        let mut tasks_to_be_scheduled = HashMap::new();
        for pre_scheduled_task in pre_scheduled_tasks {
            tasks_to_be_scheduled.insert(
                pre_scheduled_task.reference_task_name.clone(),
                pre_scheduled_task,
            );
        }

        // A new workflow does not enter this code branch
        for pending_task_ptr in pending_tasks {
            let pending_task = unsafe { pending_task_ptr.as_mut().expect("not none") };
            if SystemTaskRegistry::is_system_task(&pending_task.task_type)
                && !pending_task.status.is_terminal()
            {
                let _ = tasks_to_be_scheduled.try_insert(
                    pending_task.reference_task_name.clone(),
                    pending_task.clone(),
                );
                executed_task_ref_names.remove(&pending_task.reference_task_name);
            }

            let pending_task_ptr = pending_task as *mut TaskModel;
            let workflow_ptr = workflow as *mut WorkflowModel;
            let mut task_definition = pending_task.get_task_definition();
            if task_definition.is_none() {
                task_definition = workflow
                    .workflow_definition
                    .get_task_by_ref_name(&pending_task.reference_task_name)
                    .and_then(|x| x.task_definition.as_ref());
            }

            if let Some(task_definition) = task_definition {
                Self::check_task_timeout(task_definition, unsafe {
                    pending_task_ptr.as_mut().expect("not none")
                })?;
                Self::check_task_poll_timeout(task_definition, unsafe {
                    pending_task_ptr.as_mut().expect("not none")
                })?;
                // If the task has not been updated for "responseTimeoutSeconds" then mark task as
                // TIMED_OUT
                if Self::is_response_timeout(task_definition, &pending_task) {
                    Self::timeout_task(task_definition, unsafe {
                        pending_task_ptr.as_mut().expect("not none")
                    });
                }
            }

            if !pending_task.status.is_successful() {
                let mut workflow_task = pending_task.workflow_task.as_ref();
                if workflow_task.is_none() {
                    workflow_task = workflow
                        .workflow_definition
                        .get_task_by_ref_name(&pending_task.reference_task_name)
                }

                let retry_task = Self::retry(
                    task_definition,
                    workflow_task,
                    unsafe { pending_task_ptr.as_mut().expect("not none") },
                    unsafe { workflow_ptr.as_mut().expect("not none") },
                )?;
                if let Some(retry_task) = retry_task {
                    executed_task_ref_names.remove(&retry_task.reference_task_name);
                    tasks_to_be_scheduled
                        .insert(retry_task.reference_task_name.clone(), retry_task);
                    out_come.tasks_to_be_updated.push(pending_task);
                } else {
                    pending_task.status = TaskStatus::CompletedWithErrors;
                }
            }

            if !pending_task.executed && !pending_task.retried && pending_task.status.is_terminal()
            {
                pending_task.executed = true;
                let mut next_tasks = Self::get_next_task(workflow, &pending_task)?;
                if pending_task.iteration > 0
                    && !TaskType::DoWhile
                        .as_ref()
                        .eq(pending_task.task_type.as_str())
                    && !next_tasks.is_empty()
                {
                    next_tasks =
                        Self::filter_next_loop_over_tasks(next_tasks, &pending_task, workflow);
                }
                debug!(
                    "Scheduling Tasks from {}, next = {:?} for workflowId: {}",
                    pending_task.task_def_name,
                    next_tasks
                        .iter()
                        .map(|x| &x.task_def_name)
                        .collect::<Vec<_>>(),
                    workflow.workflow_id
                );
                for next_task in next_tasks {
                    let _ = tasks_to_be_scheduled
                        .try_insert(next_task.reference_task_name.clone(), next_task);
                }
                out_come.tasks_to_be_updated.push(pending_task);
            }
        }

        // All the tasks that need to scheduled are added to the outcome, in case of
        let un_scheduled_tasks = tasks_to_be_scheduled
            .values()
            .filter(|x| !executed_task_ref_names.contains(&x.reference_task_name))
            .map(|x| x.clone())
            .collect::<Vec<_>>();
        if !un_scheduled_tasks.is_empty() {
            debug!(
                "Scheduling Tasks: {:?} for workflow: {}",
                un_scheduled_tasks
                    .iter()
                    .map(|x| &x.task_def_name)
                    .collect::<Vec<_>>(),
                workflow.workflow_id
            );
            out_come.tasks_to_be_scheduled.extend(un_scheduled_tasks);
        }
        if has_successful_terminate_task
            || (out_come.tasks_to_be_scheduled.is_empty()
                && Self::check_for_workflow_completion(workflow)?)
        {
            debug!("Marking workflow: {:?} as complete.", workflow);
            out_come.is_complete = true;
        }

        Ok(out_come)
    }

    fn filter_next_loop_over_tasks(
        mut tasks: Vec<TaskModel>,
        pending_task: &TaskModel,
        workflow: &WorkflowModel,
    ) -> Vec<TaskModel> {
        // Update the task reference name and iteration
        tasks.iter_mut().for_each(|x| {
            TaskUtils::append_iteration(&mut x.reference_task_name, pending_task.iteration);
            x.iteration = pending_task.iteration;
        });

        let tasks_in_workflow = workflow
            .tasks
            .iter()
            .filter(|x| x.status == TaskStatus::InProgress || x.status.is_terminal())
            .map(|x| x.reference_task_name.clone())
            .collect::<HashSet<_>>();

        tasks
            .iter()
            .filter(|x| !tasks_in_workflow.contains(&x.reference_task_name))
            .map(|x| x.clone())
            .collect()
    }

    fn start_workflow(workflow: &mut WorkflowModel) -> TegResult<Vec<TaskModel>> {
        debug!("Starting workflow: {:?}", workflow);

        let workflow_def = &workflow.workflow_definition;

        // The tasks will be empty in case of new workflow
        let tasks = &mut workflow.tasks;
        // Check if the workflow is a re-run case or if it is a new workflow execution
        if workflow.re_run_from_workflow_id.is_empty() || tasks.is_empty() {
            if workflow_def.tasks.is_empty() {
                terminate_workflow_exception::STATUS
                    .with(|x| x.replace(Some(WorkflowStatus::Completed)));
                return fmt_err!(TerminateWorkflow, "No tasks found to be executed, ");
            }

            // // Nothing is running yet - so schedule the first task
            let mut task_to_schedule = workflow_def.tasks.get(0);
            // Loop until a non-skipped task is found
            while Self::is_task_skipped(task_to_schedule, workflow)? {
                task_to_schedule = workflow_def.get_next_task(
                    &task_to_schedule
                        .expect("not skip means not none")
                        .task_reference_name,
                );
            }

            // In case of a new workflow, the first non-skippable task will be scheduled
            return Self::get_tasks_to_be_scheduled(
                workflow,
                task_to_schedule.expect("not none"),
                0,
            );
        }

        // Get the first task to schedule
        if let Some(rerun_from_task) = tasks.front_mut().map(|x| {
            x.status = TaskStatus::Scheduled;
            x.retried = true;
            x.retry_count = 0;
            x
        }) {
            Ok(vec![rerun_from_task.clone()])
        } else {
            terminate_workflow_exception::STATUS.with(|x| x.take());
            terminate_workflow_exception::TASK.with(|x| x.take());
            fmt_err!(
                TerminateWorkflow,
                "The workflow {} is marked for re-run from {} but could not find the starting task",
                workflow.workflow_id,
                workflow.re_run_from_workflow_id
            )
        }
    }

    /// Updates the workflow output.
    pub fn update_workflow_output(
        workflow: &mut WorkflowModel,
        task: Option<&TaskModel>,
    ) -> TegResult<()> {
        let all_tasks = &workflow.tasks;
        if all_tasks.is_empty() {
            return Ok(());
        }

        let mut output = HashMap::new();
        if let Some(terminate_task) = all_tasks
            .iter()
            .filter(|x| {
                x.task_type.eq(TaskType::Terminate.as_ref())
                    && x.status.is_terminal()
                    && x.status.is_successful()
            })
            .collect::<VecDeque<_>>()
            .pop_front()
        {
            if !terminate_task
                .external_output_payload_storage_path
                .trim()
                .is_empty()
            {
                unimplemented!()
            } else if !terminate_task.output_data.is_empty() {
                output = terminate_task.output_data.clone();
            }
        } else {
            let last = task
                .or_else(|| all_tasks.back())
                .expect("all_task not empty");
            let workflow_def = &workflow.workflow_definition;
            if !workflow_def.output_parameters.is_empty() {
                output = ParametersUtils::get_task_input(
                    &workflow_def.output_parameters,
                    workflow,
                    None,
                    None,
                )?;
            } else if !last.external_output_payload_storage_path.trim().is_empty() {
                unimplemented!()
            } else {
                output = last.output_data.clone();
            }
        }
        workflow.output = output;
        Ok(())
    }

    fn check_for_workflow_completion(workflow: &WorkflowModel) -> TegResult<bool> {
        let mut task_status_map = HashMap::new();
        let mut non_executed_tasks = Vec::new();
        for task in &workflow.tasks {
            task_status_map.insert(task.reference_task_name.clone(), task.status);
            if !task.status.is_terminal() {
                return Ok(false);
            }

            // If there is a TERMINATE task that has been executed successfully then the workflow
            // should be marked as completed.
            if TaskType::Terminate.as_ref().eq(&task.task_type)
                && task.status.is_terminal()
                && task.status.is_successful()
            {
                return Ok(true);
            }
            if !task.retried || !task.executed {
                non_executed_tasks.push(task);
            }
        }

        // If there are no tasks executed, then we are not done yet
        if task_status_map.is_empty() {
            return Ok(false);
        }

        let workflow_tasks = &workflow.workflow_definition.tasks;
        for wf_task in workflow_tasks {
            if let Some(status) = task_status_map.get(&wf_task.task_reference_name) {
                if !status.is_terminal() {
                    return Ok(false);
                }
                // if we reach here, the task has been completed.
                // Was the task successful in completion?
                if !status.is_successful() {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }

        for wf_task in non_executed_tasks {
            if Self::get_next_tasks_to_be_scheduled(workflow, wf_task)?
                .map(|n| !task_status_map.contains_key(&n))
                .unwrap_or(false)
            {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn get_next_task(workflow: &WorkflowModel, task: &TaskModel) -> TegResult<Vec<TaskModel>> {
        let workflow_def = &workflow.workflow_definition;

        // Get the following task after the last completed task
        if SystemTaskRegistry::is_system_task(&task.task_type)
            && (TaskType::Decision.as_ref().eq(&task.task_type)
                || TaskType::Switch.as_ref().eq(&task.task_type))
        {
            if task
                .input_data
                .contains_key(&InlineStr::from("hasChildren"))
            {
                return Ok(vec![]);
            }
        }

        let task_reference_name = if task.iteration > 0 {
            TaskUtils::remove_iteration_from_task_ref_name(&task.reference_task_name)
        } else {
            task.reference_task_name.as_str()
        };
        let mut task_to_schedule = workflow_def.get_next_task(task_reference_name);
        while Self::is_task_skipped(task_to_schedule, workflow)? {
            task_to_schedule = workflow_def
                .get_next_task(&task_to_schedule.expect("not none").task_reference_name);
        }
        if let Some(task_to_schedule) = task_to_schedule {
            if TaskType::DoWhile
                .as_ref()
                .eq(task_to_schedule.type_.as_str())
            {
                // check if already has this DO_WHILE task, ignore it if it already exists
                let next_task_ref_name = &task_to_schedule.task_reference_name;
                if workflow
                    .tasks
                    .iter()
                    .any(|x| x.reference_task_name.eq(next_task_ref_name))
                {
                    return Ok(vec![]);
                }
            }
        }
        if let Some(task_to_schedule) = task_to_schedule {
            Self::get_tasks_to_be_scheduled(workflow, task_to_schedule, 0)
        } else {
            Ok(vec![])
        }
    }

    fn get_next_tasks_to_be_scheduled(
        workflow: &WorkflowModel,
        task: &TaskModel,
    ) -> TegResult<Option<InlineStr>> {
        let def = &workflow.workflow_definition;

        let task_reference_name = &task.reference_task_name;
        let mut task_to_schedule = def.get_next_task(task_reference_name);
        while Self::is_task_skipped(task_to_schedule, workflow)? {
            task_to_schedule =
                def.get_next_task(&task_to_schedule.expect("not none").task_reference_name);
        }
        Ok(task_to_schedule.map(|x| x.task_reference_name.clone()))
    }

    fn retry(
        task_def: Option<&TaskDef>,
        workflow_task: Option<&WorkflowTask>,
        task: &mut TaskModel,
        workflow: &mut WorkflowModel,
    ) -> TegResult<Option<TaskModel>> {
        let retry_count = task.retry_count;

        let task_def_guard = if task_def.is_none() {
            MetadataDao::get_task_def(&task.task_def_name)
        } else {
            None
        };
        let task_def = if let Some(task_def) = task_def {
            Some(task_def)
        } else {
            task_def_guard.as_ref().map(|x| x.value())
        };

        let expected_retry_count = if let Some(task_def) = task_def {
            workflow_task
                .map(|x| x.retry_count)
                .unwrap_or(task_def.retry_count)
        } else {
            0
        };
        if !task.status.is_retriable()
            || TaskType::is_builtin(&task.task_type)
            || expected_retry_count <= retry_count
        {
            if let Some(workflow_task) = workflow_task {
                if workflow_task.optional {
                    return Ok(None);
                }
            }
            let status = match task.status {
                TaskStatus::Canceled => WorkflowStatus::Terminated,
                TaskStatus::TimedOut => WorkflowStatus::TimedOut,
                _ => WorkflowStatus::Failed,
            };
            Self::update_workflow_output(workflow, Some(task))?;
            terminate_workflow_exception::STATUS.with(|x| x.replace(Some(status)));
            terminate_workflow_exception::TASK.with(|x| x.replace(Some(task.clone())));
            return str_err!(TerminateWorkflow, task.reason_for_incompletion);
        }

        // retry... - but not immediately - put a delay...
        let task_def = task_def.expect("task_def not none");
        let start_delay = match task_def.retry_logic {
            RetryLogic::Fixed => task_def.retry_delay_seconds,
            RetryLogic::LinearBackoff => {
                let linear_retry_delay_sec = task_def.retry_delay_seconds
                    * task_def.backoff_scale_factor
                    * (task.retry_count + 1);
                // Reset integer overflow to max value
                if linear_retry_delay_sec < 0 {
                    i32::MAX
                } else {
                    linear_retry_delay_sec
                }
            }
            RetryLogic::ExponentialBackoff => {
                let exponential_retry_delay_sec =
                    task_def.retry_delay_seconds * 2_i32.pow(task.retry_count as u32);
                // Reset integer overflow to max value
                if exponential_retry_delay_sec < 0 {
                    i32::MAX
                } else {
                    exponential_retry_delay_sec
                }
            }
        };
        task.retried = true;

        let mut rescheduled = task.clone();
        rescheduled.start_delay_in_seconds = start_delay;
        rescheduled.callback_after_seconds = start_delay as i64;
        rescheduled.retry_count = task.retry_count + 1;
        rescheduled.retried = false;
        rescheduled.task_id = IdGenerator::generate();
        rescheduled.retried_task_id = task.task_id.clone();
        rescheduled.status = TaskStatus::Scheduled;
        rescheduled.poll_count = 0;
        rescheduled.input_data = task.input_data.clone();
        rescheduled.reason_for_incompletion = InlineStr::new();
        rescheduled.sub_workflow_id = InlineStr::new();
        rescheduled.seq = 0;
        rescheduled.scheduled_time = 0;
        rescheduled.start_time = 0;
        rescheduled.end_time = 0;
        rescheduled.worker_id = InlineStr::new();

        if !task.external_input_payload_storage_path.trim().is_empty() {
            rescheduled.external_input_payload_storage_path =
                task.external_input_payload_storage_path.clone();
        } else {
            rescheduled.input_data.extend(task.input_data.clone());
        }
        if let Some(workflow_task) = workflow_task {
            let task_input = ParametersUtils::get_task_input(
                &workflow_task.input_parameters,
                workflow,
                Some(task_def),
                Some(&rescheduled.task_id),
            )?;
            rescheduled.input_data.extend(task_input);
        }
        // for the schema version 1, we do not have to recompute the inputs
        Ok(Some(rescheduled))
    }

    fn check_workflow_timeout(workflow: &WorkflowModel) -> TegResult<()> {
        let workflow_def = &workflow.workflow_definition;
        if workflow.status.is_terminal() || workflow_def.timeout_seconds <= 0 {
            return Ok(());
        }

        let timeout = workflow_def.timeout_seconds * 1000;
        let now = Utc::now().timestamp_millis();
        let elapsed_time = if workflow.last_retried_time > 0 {
            now - workflow.last_retried_time
        } else {
            now - workflow.create_time
        };

        if elapsed_time < timeout as i64 {
            return Ok(());
        }

        let reason = format!("Workflow timed out after {} seconds. Timeout configured as {} seconds. Timeout policy configured to {}",elapsed_time/1000,workflow_def.timeout_seconds,workflow_def.timeout_policy.as_ref());

        match workflow_def.timeout_policy {
            TimeoutPolicy::AlertOnly => {
                info!("{} {}", workflow.workflow_id, reason);
                // Monitors.recordWorkflowTermination(
                //         workflow.getWorkflowName(),
                //         WorkflowModel.Status.TIMED_OUT,
                //         workflow.getOwnerApp());
                Ok(())
            }
            TimeoutPolicy::TimeOutWf => {
                terminate_workflow_exception::STATUS
                    .with(|x| x.replace(Some(WorkflowStatus::TimedOut)));

                fmt_err!(TerminateWorkflow, "{}", reason)
            }
        }
    }

    fn check_task_timeout(task_def: &TaskDef, task: &mut TaskModel) -> TegResult<()> {
        if task.status.is_terminal() || task_def.timeout_seconds <= 0 || task.start_time <= 0 {
            return Ok(());
        }

        let timeout = 1000 * task_def.timeout_seconds as i64;
        let now = Utc::now().timestamp_millis();
        let elapsed_time = now - (task.start_time + (task.start_delay_in_seconds as i64) * 1000);

        if elapsed_time < timeout {
            return Ok(());
        }

        let reason = format!(
            "Task timed out after {} seconds. Timeout configured as {} seconds. Timeout policy configured to {}",
            elapsed_time / 1000,
            task_def.timeout_seconds,
            task_def.timeout_policy.as_ref()
        );
        Self::timeout_task_with_timeout_policy(reason, task_def, task)
    }

    fn check_task_poll_timeout(task_def: &TaskDef, task: &mut TaskModel) -> TegResult<()> {
        if task_def.poll_timeout_seconds <= 0 || task.status != TaskStatus::Scheduled {
            return Ok(());
        }

        let poll_timeout = 1000 * task_def.poll_timeout_seconds as i64;
        let adjusted_pool_timeout = poll_timeout + task.callback_after_seconds as i64 * 1000;
        let now = Utc::now().timestamp_millis();
        let poll_elapsed_time =
            now - (task.scheduled_time + (task.start_delay_in_seconds as i64) * 1000);

        if poll_elapsed_time < adjusted_pool_timeout {
            return Ok(());
        }

        let reason = format!(
            "Task poll timed out after {} seconds. Poll timeout configured as {} seconds. Timeout policy configured to {}",
            poll_elapsed_time / 1000,
            poll_timeout / 1000,
            task_def.timeout_policy.as_ref()
        );
        Self::timeout_task_with_timeout_policy(reason, task_def, task)
    }

    fn timeout_task_with_timeout_policy(
        reason: String,
        task_def: &TaskDef,
        task: &mut TaskModel,
    ) -> TegResult<()> {
        // Monitors.recordTaskTimeout(task.getTaskDefName());
        match task_def.timeout_policy {
            TaskTimeoutPolicy::AlertOnly => {
                info!("{}", reason);
                Ok(())
            }
            TaskTimeoutPolicy::Retry => {
                task.status = TaskStatus::TimedOut;
                task.reason_for_incompletion = reason.into();
                Ok(())
            }
            TaskTimeoutPolicy::TimeOutWf => {
                task.status = TaskStatus::TimedOut;
                task.reason_for_incompletion = reason.as_str().into();
                terminate_workflow_exception::STATUS
                    .with(|x| x.replace(Some(WorkflowStatus::TimedOut)));
                terminate_workflow_exception::TASK.with(|x| x.replace(Some(task.clone())));
                str_err!(TerminateWorkflow, reason)
            }
        }
    }

    fn is_response_timeout(task_def: &TaskDef, task: &TaskModel) -> bool {
        if task.status.is_terminal() || Self::is_async_complete_system_task(task) {
            return false;
        }

        // calculate pendingTime
        let now = Utc::now().timestamp_millis();
        let callback_time = 1000 * task.callback_after_seconds;
        let reference_time = if task.update_time > 0 {
            task.update_time
        } else {
            task.scheduled_time
        };
        let pending_time = now - (reference_time + callback_time);
        //  Monitors.recordTaskPendingTime(task.getTaskType(), task.getWorkflowType(), pendingTime);
        let threshold_ms = Properties::get_task_pending_time_threshold_sec() * 1000;
        if pending_time > threshold_ms {
            warn!(
                "Task: {} of type: {} in workflow: {}/{} is in pending state for longer than {} ms",
                task.task_id,
                task.task_type,
                task.workflow_instance_id,
                task.workflow_type,
                threshold_ms
            );
        }

        if task.status != TaskStatus::InProgress || task_def.response_timeout_seconds == 0 {
            return false;
        }

        debug!(
            "Evaluating responseTimeOut for Task: {:?}, with Task Definition: {:?}",
            task, task_def
        );
        let response_timeout = 1000 * task_def.response_timeout_seconds as i64;
        let adjusted_response_timeout = response_timeout + callback_time;
        let no_response_time = now - task.update_time;

        if no_response_time < adjusted_response_timeout {
            debug!("Current responseTime: {} has not exceeded the configured responseTimeout of {} for the Task: {:?} with Task Definition: {:?}", pending_time,response_timeout,task,task_def);
            return false;
        }

        // Monitors.recordTaskResponseTimeout(task.getTaskDefName());
        true
    }

    fn timeout_task(task_def: &TaskDef, task: &mut TaskModel) {
        let reason = format!(
            "responseTimeout: {} exceeded for the taskId: {} with Task Definition: {}",
            task_def.response_timeout_seconds, task.task_id, task.task_def_name
        );
        debug!("{}", reason);
        task.status = TaskStatus::TimedOut;
        task.reason_for_incompletion = reason.into();
    }

    pub fn get_tasks_to_be_scheduled(
        workflow: &WorkflowModel,
        task_to_schedule: &WorkflowTask,
        retry_count: i32,
    ) -> TegResult<Vec<TaskModel>> {
        Self::get_tasks_to_be_scheduled_with_retry(workflow, task_to_schedule, retry_count, "")
    }

    pub fn get_tasks_to_be_scheduled_with_retry(
        workflow: &WorkflowModel,
        task_to_schedule: &WorkflowTask,
        retry_count: i32,
        retried_task_id: &str,
    ) -> TegResult<Vec<TaskModel>> {
        debug!("before get_task_input: {:?}", workflow.input);
        let input = ParametersUtils::get_task_input(
            &task_to_schedule.input_parameters,
            workflow,
            None,
            None,
        )?;
        debug!("get_task_input: {:?}", input);

        // get tasks already scheduled (in progress/terminal) for  this workflow instance
        let tasks_in_workflow = workflow
            .tasks
            .iter()
            .filter(|x| x.status == TaskStatus::InProgress || x.status.is_terminal())
            .map(|x| &x.reference_task_name)
            .collect::<Vec<_>>();

        let task_id = IdGenerator::generate().into();
        let task_mapper_context = TaskMapperContext::new(
            workflow,
            // &task_to_schedule
            //     .task_definition
            //     .expect("task_definition not none"),
            task_to_schedule,
            input,
            retry_count,
            retried_task_id.into(),
            task_id,
        );

        // For static forks, each branch of the fork creates a join task upon completion for
        // dynamic forks, a join task is created with the fork and also with each branch of the
        // fork.
        // A new task must only be scheduled if a task, with the same reference name is not already
        // in this workflow instance
        Ok(TaskMapperRegistry::get_task_mapper(&task_to_schedule.type_)
            .get_mapped_tasks(task_mapper_context)?
            .into_iter()
            .filter(|x| !tasks_in_workflow.contains(&&x.reference_task_name))
            .collect::<Vec<_>>())
    }

    fn is_task_skipped(
        task_to_schedule: Option<&WorkflowTask>,
        workflow: &WorkflowModel,
    ) -> TegResult<bool> {
        if let Some(task_to_schedule) = task_to_schedule {
            match workflow.get_task_by_ref_name(&task_to_schedule.task_reference_name) {
                Ok(Some(t)) => {
                    if t.status == TaskStatus::Skipped {
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                }
                Ok(None) => Ok(false),
                Err(e) => {
                    str_err!(TerminateWorkflow, e.message())
                }
            }
        } else {
            Ok(false)
        }
    }

    fn is_async_complete_system_task(task: &TaskModel) -> bool {
        SystemTaskRegistry::is_system_task(&task.task_type)
            && SystemTaskRegistry::get(&task.task_type)
                .expect("not none")
                .is_async_complete(task)
    }
}

/// TODO: use RcRefCell(ref or pointer) instead cloned
pub struct DeciderOutcome {
    pub tasks_to_be_scheduled: Vec<TaskModel>,
    pub tasks_to_be_updated: Vec<*mut TaskModel>,
    pub is_complete: bool,
    pub terminate_task: Option<*mut TaskModel>,
}

impl DeciderOutcome {
    pub fn new() -> Self {
        Self {
            tasks_to_be_scheduled: Vec::default(),
            tasks_to_be_updated: Vec::default(),
            is_complete: false,
            terminate_task: None,
        }
    }
}
