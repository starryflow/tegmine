use dashmap::mapref::one::Ref;
use tegmine_common::prelude::*;
use tegmine_common::{TaskDef, TaskType, WorkflowDef, WorkflowTask};

use crate::dao::MetadataDao;
use crate::metrics::Monitors;
use crate::MetadataService;

/// Populates metadata definitions within workflow objects. Benefits of loading and populating
/// metadata definitions upfront could be:
///
/// - Immutable definitions within a workflow execution with the added benefit of guaranteeing
///   consistency at runtime.
/// - Stress is reduced on the storage layer
pub struct MetadataMapperService;

impl MetadataMapperService {
    pub fn lookup_for_workflow_definition(
        name: &InlineStr,
        version: Option<i32>,
    ) -> TegResult<(Ref<InlineStr, HashMap<i32, WorkflowDef>>, &WorkflowDef)> {
        if !MetadataService::check_workflow_def_enabled(name) {
            error!(
                "There is no enabled workflow defined with name {} and version {:?}",
                name, version
            );
            return Err(ErrorCode::NotFound(format!(
                "No such enabled workflow defined. name={}, version={:?}",
                name, version
            )));
        }

        let potential_def = if let Some(version) = version {
            Self::lookup_workflow_definition(name, version)
        } else {
            Self::lookup_latest_workflow_definition(name)
        };

        // Check if the workflow definition is valid
        potential_def
            .ok_or_else(|| {
                ErrorCode::NotFound(format!(
                    "No such workflow defined. name={}, version={:?}",
                    name, version
                ))
            })
            .inspect_err(|_| {
                error!(
                    "There is no workflow defined with name {} and version {:?}",
                    name, version
                )
            })
    }

    fn lookup_workflow_definition(
        workflow_name: &InlineStr,
        workflow_version: i32,
    ) -> Option<(Ref<InlineStr, HashMap<i32, WorkflowDef>>, &WorkflowDef)> {
        MetadataDao::get_workflow_def(workflow_name, workflow_version)
    }

    fn lookup_latest_workflow_definition(
        workflow_name: &InlineStr,
    ) -> Option<(Ref<InlineStr, HashMap<i32, WorkflowDef>>, &WorkflowDef)> {
        MetadataDao::get_latest_workflow_def(workflow_name)
    }

    // populate_workflow_with_definitions

    pub fn populate_task_definitions(workflow_definition: &mut WorkflowDef) -> TegResult<()> {
        workflow_definition.populate_tasks(Self::populate_workflow_task_with_definition);

        Self::check_not_empty_definitions(workflow_definition)
    }

    fn populate_workflow_task_with_definition(workflow_task: &mut WorkflowTask) {
        if Self::should_populate_task_definition(workflow_task) {
            workflow_task.task_definition =
                MetadataDao::get_task_def(&workflow_task.name).map(|x| x.clone());
            if workflow_task.task_definition.is_none()
                && workflow_task.type_.eq(TaskType::Simple.as_ref())
            {
                // ad-hoc task def
                workflow_task.task_definition = Some(TaskDef::new(&workflow_task.name))
            }
        }

        if workflow_task
            .type_
            .as_str()
            .eq(TaskType::SubWorkflow.as_ref())
        {
            Self::populate_version_for_sub_workflow(workflow_task)
        }
    }

    fn populate_version_for_sub_workflow(_workflow_task: &WorkflowTask) {
        unimplemented!()
    }

    fn check_not_empty_definitions(workflow_definition: &WorkflowDef) -> TegResult<()> {
        // Obtain the names of the tasks with missing definitions
        let missing_task_definition_names = workflow_definition
            .collect_tasks()
            .iter()
            .filter(|x| x.type_.eq(TaskType::Simple.as_ref()))
            .filter(|x| Self::should_populate_task_definition(x))
            .map(|x| &x.name)
            .collect::<Vec<_>>();
        if !missing_task_definition_names.is_empty() {
            error!(
                "Cannot find the task definitions for the following tasks used in workflow: {:?}",
                missing_task_definition_names
            );
            Monitors::record_workflow_start_error(
                &workflow_definition.name,
                "WorkflowContext.get().getClientApp()",
            );
            fmt_err!(
                IllegalArgument,
                "Cannot find the task definitions for the following tasks used in workflow: {:?}",
                missing_task_definition_names
            )
        } else {
            Ok(())
        }
    }

    fn should_populate_task_definition(workflow_task: &WorkflowTask) -> bool {
        workflow_task.task_definition.is_none() && !workflow_task.name.trim().is_empty()
    }
}
