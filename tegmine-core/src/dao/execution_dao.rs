use chrono::Utc;
use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use tegmine_common::prelude::*;

use crate::model::{TaskModel, TaskStatus, WorkflowModel};

/// Data access layer for storing workflow executions
pub struct ExecutionDao;

/// ******************************************
/// *************** Task *********************
/// ******************************************

// TASK_LIMIT_BUCKET

static IN_PROGRESS_TASKS: Lazy<DashMap<InlineStr, Vec<InlineStr>>> = Lazy::new(|| DashMap::new());

static TASKS_IN_PROGRESS_STATUS: Lazy<DashMap<InlineStr, Vec<InlineStr>>> =
    Lazy::new(|| DashMap::new());

static WORKFLOW_TO_TASKS: Lazy<DashMap<InlineStr, Vec<InlineStr>>> = Lazy::new(|| DashMap::new());

static SCHEDULED_TASKS: Lazy<DashMap<InlineStr, HashMap<InlineStr, InlineStr>>> =
    Lazy::new(|| DashMap::new());

static TASK: Lazy<DashMap<InlineStr, TaskModel>> = Lazy::new(|| DashMap::new());

/// ******************************************
/// *************** Workflow *****************
/// ******************************************

static WORKFLOW: Lazy<DashMap<InlineStr, WorkflowModel>> = Lazy::new(|| DashMap::new());

static PENDING_WORKFLOWS: Lazy<DashMap<InlineStr, Vec<InlineStr>>> = Lazy::new(|| DashMap::new());

static WORKFLOW_DEF_TO_WORKFLOWS: Lazy<DashMap<(InlineStr, i64), Vec<InlineStr>>> =
    Lazy::new(|| DashMap::new());

static CORR_ID_TO_WORKFLOWS: Lazy<DashMap<InlineStr, Vec<InlineStr>>> =
    Lazy::new(|| DashMap::new());

// EVENT_EXECUTION

impl ExecutionDao {
    /// ******************************************
    /// *************** Task *********************
    /// ******************************************

    // getPendingTasksByWorkflow

    // getTasks

    pub fn create_tasks(
        tasks: Vec<TaskModel>,
    ) -> TegResult<Vec<Ref<'static, InlineStr, TaskModel>>> {
        let mut task_refs = Vec::with_capacity(tasks.len());

        for mut task in tasks {
            Self::validate(&task)?;

            let task_key = task.get_task_key();

            let added = SCHEDULED_TASKS
                .entry(task.workflow_instance_id.clone())
                .or_insert(HashMap::default())
                .insert(task_key.clone(), task.task_id.clone())
                .is_some();
            if !added {
                debug!(
                    "Task already scheduled, skipping the run {}, ref={}, key={}",
                    task.task_id, task.reference_task_name, task_key
                );
                continue;
            }

            if !task.status.is_terminal() && task.scheduled_time == 0 {
                task.scheduled_time = Utc::now().timestamp_millis();
            }

            Self::correlate_task_to_workflow_in_ds(&task.task_id, &task.workflow_instance_id);
            debug!(
                "Scheduled task added to WORKFLOW_TO_TASKS workflowId: {}, taskId: {}, taskType: {} during createTasks",
                task.workflow_instance_id, task.task_id, task.task_type
            );

            IN_PROGRESS_TASKS
                .entry(task.task_def_name.clone())
                .or_default()
                .push(task.task_id.clone());
            debug!(
                "Scheduled task added to IN_PROGRESS_TASKS with inProgressTaskKey: {}, workflowId: {}, taskId: {}, taskType: {} during createTasks",
                task.task_def_name, task.workflow_instance_id, task.task_id, task.task_type
            );

            task_refs.push(Self::update_task(task));
        }

        Ok(task_refs)
    }

    pub fn update_task(task: TaskModel) -> Ref<'static, InlineStr, TaskModel> {
        let task_id = task.task_id.clone();
        let task_definition = task.get_task_definition();

        if task_definition.map(|x| x.concurrency_limit()).unwrap_or(0) > 0 {
            if task.status == TaskStatus::InProgress {
                TASKS_IN_PROGRESS_STATUS
                    .entry(task.task_def_name.clone())
                    .or_default()
                    .push(task_id.clone());
                debug!(
                    "Workflow Task added to TASKS_IN_PROGRESS_STATUS with tasksInProgressKey: {}, workflowId: {}, taskId: {}, taskType: {}, taskStatus: {} during updateTask",
                    task.task_def_name,
                    task.workflow_instance_id,
                    task_id,
                    task.task_type,
                    task.status.as_ref()
                );
            } else {
                TASKS_IN_PROGRESS_STATUS
                    .entry(task.task_def_name.clone())
                    .or_default()
                    .retain(|x| !x.eq(&task_id));
                debug!(
                    "Workflow Task removed from TASKS_IN_PROGRESS_STATUS with tasksInProgressKey: {}, workflowId: {}, taskId: {}, taskType: {}, taskStatus: {} during updateTask",
                    task.task_def_name,
                    task.workflow_instance_id,
                    task_id,
                    task.task_type,
                    task.status.as_ref()
                );

                // TODO: TASK_LIMIT_BUCKET
                debug!(
                    "Workflow Task removed from TASK_LIMIT_BUCKET with taskLimitBucketKey: {}, workflowId: {}, taskId: {}, taskType: {}, taskStatus: {} during updateTask",
                    task.task_def_name,
                    task.workflow_instance_id,
                    task_id,
                    task.task_type,
                    task.status.as_ref()
                );
            }
        }

        TASK.insert(task_id.clone(), task);
        let task = TASK.get(&task_id).expect("not none");
        debug!(
            "Workflow task payload saved to TASK with taskKey: {}, workflowId: {}, taskId: {}, taskType: {} during updateTask",
            task.task_id, task.workflow_instance_id, task.task_id, task.task_type
        );
        if task.status.is_terminal() {
            IN_PROGRESS_TASKS
                .entry(task.task_def_name.clone())
                .or_default()
                .retain(|x| !x.eq(&task.task_id));
            debug!(
                "Workflow Task removed from TASKS_IN_PROGRESS_STATUS with tasksInProgressKey: {}, workflowId: {}, taskId: {}, taskType: {}, taskStatus: {} during updateTask",
                task.task_def_name,
                task.workflow_instance_id,
                task.task_id,
                task.task_type,
                task.status.as_ref()
            );
        }

        let task_ids = WORKFLOW_TO_TASKS
            .entry(task.workflow_instance_id.clone())
            .or_default();
        if !task_ids.contains(&task.task_id) {
            Self::correlate_task_to_workflow_in_ds(&task.task_id, &task.workflow_instance_id);
        }

        task
    }

    // exceedsLimit

    fn remove_task_mappings(task: &TaskModel) {
        let task_key = task.get_task_key();
        SCHEDULED_TASKS
            .get_mut(&task.workflow_instance_id)
            .map(|mut x| x.remove(&task_key));
        IN_PROGRESS_TASKS
            .get_mut(&task.task_def_name)
            .map(|mut x| x.retain(|x| !x.eq(&task.task_id)));
        WORKFLOW_TO_TASKS
            .get_mut(&task.workflow_instance_id)
            .map(|mut x| x.retain(|x| !x.eq(&task.task_id)));
        TASKS_IN_PROGRESS_STATUS
            .get_mut(&task.task_def_name)
            .map(|mut x| x.retain(|x| !x.eq(&task.task_id)));
        // TASK_LIMIT_BUCKET
    }

    // removeTaskMappingsWithExpiry

    pub fn remove_task(task_id: &InlineStr) -> bool {
        if let Some(task) = Self::get_task(task_id) {
            Self::remove_task_mappings(&task);
            let _ = TASK.remove(task_id);
            true
        } else {
            warn!("No such task found by id {}", task_id);
            false
        }
    }

    // removeTaskWithExpiry

    pub fn get_task(task_id: &InlineStr) -> Option<Ref<InlineStr, TaskModel>> {
        TASK.get(task_id)
    }

    pub fn get_tasks(task_ids: Vec<InlineStr>) -> Vec<TaskModel> {
        let mut tasks = Vec::with_capacity(task_ids.len());
        for task_id in &task_ids {
            if let Some(task) = TASK.get(task_id) {
                tasks.push(task.value().clone());
            }
        }
        tasks
    }

    pub fn get_tasks_ref(task_ids: Vec<InlineStr>) -> Vec<Ref<'static, InlineStr, TaskModel>> {
        let mut tasks = Vec::with_capacity(task_ids.len());
        for task_id in &task_ids {
            if let Some(task) = TASK.get(task_id) {
                tasks.push(task);
            }
        }
        tasks
    }

    pub fn get_tasks_for_workflow(workflow_id: &InlineStr) -> Vec<TaskModel> {
        let task_ids = WORKFLOW_TO_TASKS
            .get(workflow_id)
            .map(|x| x.value().clone())
            .unwrap_or_default();
        Self::get_tasks(task_ids)
    }

    pub fn get_tasks_for_workflow_ref(
        workflow_id: &InlineStr,
    ) -> Vec<Ref<'static, InlineStr, TaskModel>> {
        let task_ids = WORKFLOW_TO_TASKS
            .get(workflow_id)
            .map(|x| x.value().clone())
            .unwrap_or_default();
        Self::get_tasks_ref(task_ids)
    }

    // getPendingTasksForTaskType

    /// ******************************************
    /// *************** Workflow *****************
    /// ******************************************

    /// return Id of the newly created workflow
    pub fn create_workflow(workflow: WorkflowModel) -> Ref<'static, InlineStr, WorkflowModel> {
        Self::insert_or_update_workflow(workflow, false)
    }

    pub fn update_workflow(workflow: WorkflowModel) -> Ref<'static, InlineStr, WorkflowModel> {
        Self::insert_or_update_workflow(workflow, true)
    }

    /// return true if the deletion is successful, false otherwise
    pub fn remove_workflow(workflow_id: &InlineStr) -> bool {
        if let Some((workflow, tasks)) = Self::get_workflow_ref(workflow_id) {
            // Remove from lists
            WORKFLOW_DEF_TO_WORKFLOWS
                .get_mut(&(
                    workflow.workflow_definition.name.clone(),
                    workflow.create_time,
                ))
                .map(|mut x| x.value_mut().retain(|x| !x.eq(workflow_id)));
            CORR_ID_TO_WORKFLOWS
                .get_mut(&workflow.correlation_id)
                .map(|mut x| x.value_mut().retain(|x| !x.eq(workflow_id)));
            PENDING_WORKFLOWS
                .get_mut(&workflow.workflow_definition.name)
                .map(|mut x| x.value_mut().retain(|x| !x.eq(workflow_id)));

            // Remove the object
            let _ = WORKFLOW.remove(workflow_id);

            // Remove task
            for task in &tasks {
                Self::remove_task(&task.task_id);
            }
            true
        } else {
            false
        }
    }

    // removeWorkflowWithExpiry

    pub fn remove_from_pending_workflow(workflow_type: &InlineStr, workflow_id: &InlineStr) {
        SCHEDULED_TASKS.remove(workflow_id);
        PENDING_WORKFLOWS
            .get_mut(workflow_type)
            .map(|mut x| x.retain(|x| !x.eq(workflow_id)));
    }

    #[allow(unused)]
    pub fn get_workflow(workflow_id: &InlineStr) -> Option<WorkflowModel> {
        Self::get_workflow_include_tasks(workflow_id, true)
    }

    pub fn get_workflow_ref(
        workflow_id: &InlineStr,
    ) -> Option<(
        Ref<InlineStr, WorkflowModel>,
        Vec<Ref<InlineStr, TaskModel>>,
    )> {
        Self::get_workflow_include_tasks_ref(workflow_id, true)
    }

    pub fn get_workflow_include_tasks(
        workflow_id: &InlineStr,
        include_tasks: bool,
    ) -> Option<WorkflowModel> {
        if let Some(workflow) = WORKFLOW.get(workflow_id) {
            let mut workflow_mut = workflow.clone();
            if include_tasks {
                let mut tasks = Self::get_tasks_for_workflow(workflow_id);
                tasks.sort_by(|a, b| a.seq.cmp(&b.seq));
                workflow_mut.tasks = tasks;
            }
            Some(workflow_mut)
        } else {
            None
        }
    }

    pub fn get_workflow_include_tasks_ref(
        workflow_id: &InlineStr,
        include_tasks: bool,
    ) -> Option<(
        Ref<'static, InlineStr, WorkflowModel>,
        Vec<Ref<'static, InlineStr, TaskModel>>,
    )> {
        if let Some(workflow) = WORKFLOW.get(workflow_id) {
            let tasks = if include_tasks {
                let mut tasks = Self::get_tasks_for_workflow_ref(workflow_id);
                tasks.sort_by(|a, b| a.seq.cmp(&b.seq));
                tasks
            } else {
                Vec::default()
            };

            Some((workflow, tasks))
        } else {
            None
        }
    }

    // getRunningWorkflowIds

    // getPendingWorkflowsByType

    // getWorkflowsByType

    // getWorkflowsByCorrelationId

    // canSearchAcrossWorkflows

    /// Inserts a new workflow/ updates an existing workflow in the datastore. Additionally, if a
    /// workflow is in terminal state, it is removed from the set of pending workflows.
    fn insert_or_update_workflow(
        mut workflow: WorkflowModel,
        update: bool,
    ) -> Ref<'static, InlineStr, WorkflowModel> {
        let workflow_id = workflow.workflow_id.clone();

        // Store the workflow object
        workflow.tasks.clear();
        WORKFLOW.insert(workflow_id.clone(), workflow);
        let workflow = WORKFLOW.get(&workflow_id).expect("always not empty");

        if !update {
            // Add to list of workflows for a workflowdef
            WORKFLOW_DEF_TO_WORKFLOWS
                .entry((
                    workflow.workflow_definition.name.clone(),
                    workflow.create_time,
                ))
                .or_default()
                .push(workflow_id.clone());
            if !workflow.correlation_id.is_empty() {
                // Add to list of workflows for a correlationId
                CORR_ID_TO_WORKFLOWS
                    .entry(workflow.correlation_id.clone())
                    .or_default()
                    .push(workflow_id.clone());
            }
        }

        // Add or remove from the pending workflows
        if workflow.status.is_terminal() {
            PENDING_WORKFLOWS
                .get_mut(&workflow.workflow_definition.name)
                .map(|mut x| x.value_mut().retain(|x| !x.eq(&workflow_id)));
        } else {
            PENDING_WORKFLOWS
                .entry(workflow.workflow_definition.name.clone())
                .or_default()
                .push(workflow_id.clone())
        }

        workflow
    }

    /// Stores the correlation of a task to the workflow instance in the datastore
    fn correlate_task_to_workflow_in_ds(task_id: &InlineStr, workflow_instance_id: &InlineStr) {
        WORKFLOW_TO_TASKS
            .entry(workflow_instance_id.clone())
            .or_default()
            .push(task_id.clone());
        debug!(
            "Task mapped in WORKFLOW_TO_TASKS with  workflowId: {}, taskId: {}",
            workflow_instance_id, task_id
        )
    }

    // getPendingWorkflowCount

    // getInProgressTaskCount

    /// ******************************************
    /// *************** Event *****************
    /// ******************************************

    // addEventExecution

    // updateEventExecution

    // removeEventExecution

    // getEventExecutions

    fn validate(task: &TaskModel) -> TegResult<()> {
        if task.task_id.is_empty() {
            fmt_err!(IllegalArgument, "task object cannot be null")
        } else if task.workflow_instance_id.is_empty() {
            fmt_err!(IllegalArgument, "Workflow instance id cannot be null")
        } else if task.reference_task_name.is_empty() {
            fmt_err!(IllegalArgument, "Task reference name cannot be null")
        } else {
            Ok(())
        }
    }
}