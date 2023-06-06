use chrono::Utc;
use tegmine_common::prelude::*;
use tegmine_common::{TaskDef, WorkflowDef};

use crate::dao::MetadataDao;

pub struct MetadataService;

impl MetadataService {
    /// ******************************************
    /// *************** Workflow *****************
    /// ******************************************

    /// The Workflow Definition contains all the information necessary to define the behavior of a
    /// workflow.
    pub fn register_workflow_def(mut workflow_def: WorkflowDef) -> TegResult<()> {
        workflow_def.create_time = Utc::now().timestamp_millis();
        MetadataDao::create_workflow_def(workflow_def)
    }

    pub fn update_workflow_def(mut workflow_def: WorkflowDef) {
        workflow_def.update_time = Utc::now().timestamp_millis();
        MetadataDao::update_workflow_def(workflow_def)
    }

    pub fn unregister_workflow_def(name: &InlineStr, version: i32) -> TegResult<()> {
        MetadataDao::remove_workflow_def(name, version)
    }

    pub fn toggle_workflow_def(name: &InlineStr, enable: bool)->TegResult<()> {
        MetadataDao::toggle_workflow_def(name, enable)
    }

    pub fn check_workflow_def_enabled(name: &InlineStr) -> bool {
        MetadataDao::check_workflow_def_endabled(name)
    }

    /// ******************************************
    /// *************** Task ********************
    /// ******************************************

    /// Task Definitions are used to register SIMPLE tasks (workers).
    pub fn register_task_def(task_defs: Vec<TaskDef>, client_app: &str) -> TegResult<()> {
        for mut task_def in task_defs {
            task_def.created_by = client_app.into();
            task_def.create_time = Utc::now().timestamp_millis();
            task_def.updated_by = InlineStr::default();
            task_def.update_time = 0;
            MetadataDao::create_task_def(task_def);
        }
        Ok(())
    }

    pub fn update_task_def(mut task_def: TaskDef, client_app: &str) -> TegResult<()> {
        let existing = MetadataDao::get_task_def(&task_def.name);
        if existing.is_none() {
            fmt_err!(NotFound, "No such task by name {}", task_def.name)
        } else {
            task_def.updated_by = client_app.into();
            task_def.update_time = Utc::now().timestamp_millis();
            MetadataDao::update_task_def(task_def);
            Ok(())
        }
    }
}
