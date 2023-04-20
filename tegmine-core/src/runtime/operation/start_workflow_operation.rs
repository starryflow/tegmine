use numtoa::NumToA;
use tegmine_common::prelude::*;
use tegmine_common::WorkflowDef;

use crate::metrics::Monitors;
use crate::model::WorkflowModel;
use crate::runtime::dal::ExecutionDaoFacade;
use crate::runtime::event::{WorkflowCreationEvent, WorkflowEvaluationEvent};
use crate::runtime::execution::{StartWorkflowInput, EVAL_EVENT_CHANNEL};
use crate::runtime::metadata::MetadataMapperService;
use crate::utils::{IdGenerator, ParametersUtils};

pub struct StartWorkflowOperation;

impl StartWorkflowOperation {
    pub fn execute(input: StartWorkflowInput) -> TegResult<InlineStr> {
        Self::start_workflow(input)
    }

    pub fn handle_workflow_creation_event(
        workflow_creation_event: WorkflowCreationEvent,
    ) -> TegResult<()> {
        Self::start_workflow(workflow_creation_event.start_workflow_input)?;
        Ok(())
    }

    fn start_workflow(mut input: StartWorkflowInput) -> TegResult<InlineStr> {
        let mut workflow_definition = if let Some(workflow_def) = input.workflow_definition.take() {
            workflow_def
        } else {
            MetadataMapperService::lookup_for_workflow_definition(&input.name, input.version)?
                .1
                .clone()
        };

        MetadataMapperService::populate_task_definitions(&mut workflow_definition)?;

        // perform validations
        let mut workflow_input = std::mem::take(&mut input.workflow_input);
        let external_input_payload_storage_path =
            std::mem::take(&mut input.external_input_payload_storage_path);
        Self::validate_workflow(
            &workflow_definition,
            &workflow_input,
            external_input_payload_storage_path.as_ref(),
        )?;

        // Generate ID if it's not present
        let workflow_id = if input.workflow_id.is_empty() {
            IdGenerator::generate()
        } else {
            input.workflow_id.clone()
        };

        // Persist the Workflow
        let mut workflow = WorkflowModel::new(workflow_id.clone(), workflow_definition, input);

        if !workflow_input.is_empty() {
            ParametersUtils::get_workflow_input(&workflow.workflow_definition, &mut workflow_input);
            workflow.input = workflow_input;
        } else {
            workflow.external_input_payload_storage_path = external_input_payload_storage_path;
        }

        let (workflow_name, workflow_version, owner_app) = (
            workflow.workflow_definition.name.clone(),
            workflow.workflow_definition.version,
            workflow.owner_app.clone(),
        );
        match Self::create_and_evaluate(workflow) {
            Ok(_) => {
                Monitors::record_workflow_start_success(
                    &workflow_name,
                    workflow_version.numtoa_str(10, &mut [0; 16]),
                    &owner_app,
                );
                Ok(workflow_id.into())
            }
            Err(e) => {
                Monitors::record_workflow_start_error(
                    &workflow_name,
                    "WorkflowContext.get().getClientApp()",
                );

                error!("Unable to start workflow: {}, error: {}", workflow_name, e);

                // It's possible the remove workflow call hits an exception as well, in that case we
                // want to log both errors to help diagnosis.
                if let Err(e) = ExecutionDaoFacade::remove_workflow(&workflow_id, false) {
                    error!(
                        "Could not remove the workflowId: {}, error: {}",
                        workflow_id, e
                    );
                }
                Err(e)
            }
        }
    }

    /// Acquire and hold the lock till the workflow creation action is completed (in primary and
    /// secondary datastores).
    ///
    /// This is to ensure that workflow creation action precedes any other action on a given
    /// workflow.
    fn create_and_evaluate(mut workflow: WorkflowModel) -> TegResult<()> {
        // executionLockService.acquireLock(workflow.getWorkflowId()))

        ExecutionDaoFacade::create_workflow(&mut workflow);
        debug!(
            "A new instance of workflow: {} created with id: {}",
            &workflow.workflow_definition.name, workflow.workflow_id
        );

        ExecutionDaoFacade::populate_workflow_and_task_payload_data(&mut workflow);

        EVAL_EVENT_CHANNEL
            .0
            .send(WorkflowEvaluationEvent::new(workflow))?;

        // executionLockService.releaseLock(workflow.getWorkflowId());
        Ok(())
    }

    /// Performs validations for starting a workflow
    fn validate_workflow(
        workflow_def: &WorkflowDef,
        workflow_input: &HashMap<InlineStr, Object>,
        external_storage_path: &str,
    ) -> TegResult<()> {
        // Check if the input to the workflow is not null
        if workflow_input.is_empty() && external_storage_path.trim().is_empty() {
            error!(
                "The input for the workflow '{}' cannot be NULL",
                &workflow_def.name
            );
            Monitors::record_workflow_start_error(
                &workflow_def.name,
                "WorkflowContext.get().getClientApp()",
            );
            str_err!(IllegalArgument, "NULL input passed when starting workflow")
        } else {
            Ok(())
        }
    }
}
