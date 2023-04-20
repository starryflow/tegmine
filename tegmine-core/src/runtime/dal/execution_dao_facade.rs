use chrono::Utc;
use tegmine_common::prelude::*;
use tegmine_common::{TaskDef, TaskExecLog};

use crate::config::Properties;
use crate::dao::{
    ConcurrentExecutionLimitDao, ExecutionDao, IndexDao, PollDataDao, QueueDao, RateLimitingDao,
};
use crate::model::{TaskModel, TaskSummary, Workflow, WorkflowModel, WorkflowSummary};
use crate::utils::QueueUtils;
use crate::WorkflowStatus;

/// Service that acts as a facade for accessing execution data from the `ExecutionDao`,
/// `RateLimitingDao` and `IndexDao` storage layers
pub struct ExecutionDaoFacade;

impl ExecutionDaoFacade {
    /// ******************************************
    /// *************** Workflow *****************
    /// ******************************************
    pub fn get_workflow_model(
        workflow_id: &InlineStr,
        include_task: bool,
    ) -> TegResult<WorkflowModel> {
        let mut workflow_model =
            Self::get_workflow_model_from_data_store(workflow_id, include_task)?;
        Self::populate_workflow_and_task_payload_data(&mut workflow_model);
        Ok(workflow_model)
    }

    pub fn get_workflow_status(workflow_id: &InlineStr) -> Option<WorkflowStatus> {
        ExecutionDao::get_workflow_status(workflow_id)
    }

    /// Fetches the `Workflow` object from the data store given the id. Attempts to fetch from
    /// `ExecutionDAO` first, if not found, attempts to fetch from `IndexDAO`.
    pub fn get_workflow(workflow_id: &InlineStr, include_task: bool) -> TegResult<Workflow> {
        Ok(Workflow::new(Self::get_workflow_model_from_data_store(
            workflow_id,
            include_task,
        )?))
    }

    // get_workflow_include_tasks

    fn get_workflow_model_from_data_store(
        workflow_id: &InlineStr,
        include_task: bool,
    ) -> TegResult<WorkflowModel> {
        if let Some(workflow) = ExecutionDao::get_workflow_include_tasks(workflow_id, include_task)
        {
            Ok(workflow)
        } else {
            // read from indexDao
            // if not exist, return Error
            // unimplemented!()
            fmt_err!(NotFound, "No such workflow found by id: {}", workflow_id)
        }
    }

    // getWorkflowsByCorrelationId

    // getWorkflowsByName

    // getPendingWorkflowsByName

    // getRunningWorkflowIds

    // getPendingWorkflowCount

    /// Creates a new workflow in the data store
    /// return the id of the created workflow
    pub fn create_workflow(workflow_model: &mut WorkflowModel) {
        Self::externalize_workflow_data(workflow_model);
        ExecutionDao::create_workflow(workflow_model);

        // Add to decider queue
        QueueDao::push(
            QueueDao::DECIDER_QUEUE,
            &workflow_model.workflow_id,
            workflow_model.priority,
            Properties::default().workflow_offset_timeout_sec,
        );
        if Properties::default().async_indexing_enabled {
            IndexDao::async_index_workflow(WorkflowSummary::new(workflow_model));
        } else {
            IndexDao::index_workflow(WorkflowSummary::new(workflow_model));
        }
    }

    /// Updates the given workflow in the data store
    pub fn update_workflow(workflow_model: &mut WorkflowModel) {
        workflow_model.updated_time = Utc::now().timestamp_millis();
        if workflow_model.status.is_terminal() {
            workflow_model.end_time = Utc::now().timestamp_millis();
        }
        Self::externalize_workflow_data(&workflow_model);
        ExecutionDao::update_workflow(workflow_model);
        if Properties::default().async_indexing_enabled {
            unimplemented!()
        } else {
            IndexDao::index_workflow(WorkflowSummary::new(workflow_model));
        }
    }

    fn externalize_workflow_data(_workflow_model: &WorkflowModel) {
        // external_payload_storage_utils.verify_and_upload(workflow_model,
        // PayloadType.WORKFLOW_INPUT);
        // external_payload_storage_utils.verify_and_upload(workflow_model,
        // PayloadType.WORKFLOW_OUTPUT);
    }

    pub fn remove_from_pending_workflow(workflow_type: &InlineStr, workflow_id: &InlineStr) {
        ExecutionDao::remove_from_pending_workflow(workflow_type, workflow_id);
    }

    /// Removes the workflow from the data store.
    pub fn remove_workflow(workflow_id: &InlineStr, archive_workflow: bool) -> TegResult<()> {
        let workflow = Self::get_workflow_model_from_data_store(workflow_id, true)?;

        ExecutionDao::remove_workflow(workflow_id);

        // TODO:
        if let Err(_e) = Self::remove_workflow_index(&workflow, archive_workflow) {
            unimplemented!()
        }
        // removeTaskIndex

        if let ControlFlow::Break(e) = workflow.tasks.iter().try_for_each(|x| {
            if let Err(_e) = Self::remove_task_index(&workflow, x, archive_workflow) {
                unimplemented!()
            }

            if let Err(e) =
                QueueDao::remove(&QueueUtils::get_queue_name_by_task_model(x), &x.task_id)
            {
                info!(
                    "Error removing task: {} of workflow: {} from {} queue, error: {}",
                    workflow_id,
                    x.task_id,
                    QueueUtils::get_queue_name_by_task_model(x),
                    e
                )
            }

            ControlFlow::Continue(())
        }) {
            return Err(e);
        }

        if let Err(e) = QueueDao::remove(QueueDao::DECIDER_QUEUE, workflow_id) {
            info!(
                "Error removing workflow: {} from decider queue, error: {}",
                workflow_id, e
            );
        }
        Ok(())
    }

    fn remove_workflow_index(_workflow: &WorkflowModel, _archive_workflow: bool) -> TegResult<()> {
        // TODO:
        Ok(())
    }

    // removeWorkflowWithExpiry

    // resetWorkflow

    /// ******************************************
    /// *************** Task *********************
    /// ******************************************

    // getTasksForWorkflow

    pub fn get_task_model(task_id: &InlineStr) -> Option<TaskModel> {
        let task_model = Self::get_task_from_datastore(task_id);
        if let Some(task_model) = &task_model {
            Self::populate_task_data(task_model);
        }
        task_model
    }

    // getTask

    // getTaskFromDatastore
    fn get_task_from_datastore(task_id: &InlineStr) -> Option<TaskModel> {
        ExecutionDao::get_task(task_id)
    }

    // getTasksByName

    // getPendingTasksForTaskType

    // getInProgressTaskCount

    pub fn create_tasks(tasks: &mut [&mut TaskModel]) -> TegResult<()> {
        tasks.iter().for_each(|x| Self::externalize_task_data(x));
        ExecutionDao::create_tasks(tasks)
    }

    pub fn update_tasks(tasks: &[*mut TaskModel]) {
        tasks.iter().for_each(|&x| {
            let _ = Self::update_task(from_addr_mut!(x));
        });
    }

    /// Sets the update time for the task. Sets the end time for the task (if task is in terminal
    /// state and end time is not set). Updates the task in the `ExecutionDao` first, then stores it
    /// in the `IndexDao`.
    pub fn update_task(task_model: &mut TaskModel) -> TegResult<()> {
        if !task_model.status.is_terminal()
            || (task_model.status.is_terminal() && task_model.update_time == 0)
        {
            task_model.update_time = Utc::now().timestamp_millis();
        }
        if task_model.status.is_terminal() && task_model.end_time == 0 {
            task_model.end_time = Utc::now().timestamp_millis();
        }

        Self::externalize_task_data(&task_model);
        ExecutionDao::update_task(task_model)?;

        // Indexing a task for every update adds a lot of volume. That is ok but if async indexing
        // is enabled and tasks are stored in memory until a block has completed, we would lose a
        // lot of tasks on a system failure. So only index for each update if async indexing is not
        // enabled. If it *is* enabled, tasks will be indexed only when a workflow is in
        // terminal state.
        if !Properties::default().async_indexing_enabled {
            IndexDao::index_task(TaskSummary::new(task_model));
        }
        Ok(())
    }

    fn externalize_task_data(_task_model: &TaskModel) {
        // external_payload_storage_utils.verify_and_upload(task_model,
        // PayloadType.TASK_INPUT);
        // external_payload_storage_utils.verify_and_upload(task_model,
        // PayloadType.TASK_OUTPUT);
    }

    // removeTask

    fn remove_task_index(
        _workflow: &WorkflowModel,
        _task: &TaskModel,
        _archive_task: bool,
    ) -> TegResult<()> {
        // TODO
        Ok(())
    }

    pub fn extend_lease(task_model: &mut TaskModel) -> TegResult<()> {
        task_model.update_time = Utc::now().timestamp_millis();
        ExecutionDao::update_task(task_model)
    }

    // getTaskPollData

    // getAllPollData

    // getTaskPollDataByDomain
    // pub fn get_task_poll_data_by_domain(task_name:&InlineStr, domain:&str) -> PollData{

    // }

    /// ******************************************
    /// *************** Event ********************
    /// ******************************************

    /// ******************************************
    /// *************** Other ********************
    /// ******************************************

    pub fn update_task_last_poll(task_name: &str, domain: &str, worker_id: &str) {
        if let Err(e) = PollDataDao::update_last_poll_data(task_name, domain, worker_id) {
            error!(
                "Error updating PollData for task: {} in domain: {} from worker: {}, error: {}",
                task_name, domain, worker_id, e
            );
            // Monitors.error(this.getClass().getCanonicalName(), "updateTaskLastPoll");
        }
    }

    pub fn exceeds_in_progress_limit(task: &TaskModel) -> bool {
        ConcurrentExecutionLimitDao::exceeds_limit(task)
    }

    pub fn exceeds_rate_limit_per_frequency(task: &TaskModel, task_def: Option<&TaskDef>) -> bool {
        RateLimitingDao::exceeds_rate_limit_per_frequency(task, task_def)
    }

    pub fn add_task_exec_log(_logs: Vec<TaskExecLog>) {
        unimplemented!()
    }

    /// Populates the workflow input data and the tasks input/output data if stored in external
    /// payload storage.
    pub fn populate_workflow_and_task_payload_data(workflow_model: &mut WorkflowModel) {
        if !workflow_model
            .external_input_payload_storage_path
            .trim()
            .is_empty()
        {
            unimplemented!()
        }

        if !workflow_model
            .external_output_payload_storage_path
            .trim()
            .is_empty()
        {
            unimplemented!()
        }

        workflow_model
            .tasks
            .iter_mut()
            .for_each(|x| Self::populate_task_data(x));
    }

    pub fn populate_task_data(task_model: &TaskModel) {
        if !task_model
            .external_output_payload_storage_path
            .trim()
            .is_empty()
        {
            unimplemented!()
        }

        if !task_model
            .external_input_payload_storage_path
            .trim()
            .is_empty()
        {
            unimplemented!()
        }
    }
}
