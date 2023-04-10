use dashmap::mapref::multiple::RefMulti;
use dashmap::mapref::one::Ref;
use dashmap::setref::multiple::RefMulti as SetRefMulti;
use dashmap::{DashMap, DashSet};
use once_cell::sync::Lazy;
use tegmine_common::prelude::*;
use tegmine_common::{TaskDef, WorkflowDef};

/// Data access layer for the workflow metadata - task definitions and workflow definitions
pub struct MetadataDao;

static TASK_DEF: Lazy<DashMap<InlineStr, TaskDef>> = Lazy::new(|| DashMap::new());

static WORKFLOW_DEF: Lazy<DashMap<InlineStr, HashMap<i32, WorkflowDef>>> =
    Lazy::new(|| DashMap::new());

static WORKFLOW_DEF_NAMES: Lazy<DashSet<InlineStr>> = Lazy::new(|| DashSet::new());

impl MetadataDao {
    /// ******************************************
    /// *************** TaskDef **************
    /// ******************************************
    pub fn create_task_def(task_def: TaskDef) {
        Self::insert_or_update_task_def(task_def)
    }

    pub fn update_task_def(task_def: TaskDef) {
        Self::insert_or_update_task_def(task_def)
    }

    fn insert_or_update_task_def(task_def: TaskDef) {
        let task_name = task_def.name.clone();

        // Store all task def in under one key
        TASK_DEF.insert(task_name.clone(), task_def);
    }

    pub fn get_task_def(name: &InlineStr) -> Option<Ref<'static, InlineStr, TaskDef>> {
        TASK_DEF.get(name)
    }

    #[allow(unused)]
    pub fn get_all_task_defs() -> Vec<RefMulti<'static, InlineStr, TaskDef>> {
        TASK_DEF.iter().collect::<Vec<_>>()
    }

    #[allow(unused)]
    pub fn remove_task_def(name: &InlineStr) -> TegResult<()> {
        if let None = TASK_DEF.remove(name) {
            fmt_err!(
                NotFound,
                "Cannot remove the task: {} - no such task definition",
                name
            )
        } else {
            Ok(())
        }
    }

    /// ******************************************
    /// *************** WorkflowDef **************
    /// ******************************************

    pub fn create_workflow_def(workflow_def: WorkflowDef) -> TegResult<()> {
        if WORKFLOW_DEF
            .get(&workflow_def.name)
            .map(|x| x.value().contains_key(&workflow_def.version))
            .unwrap_or(false)
        {
            return fmt_err!(
                Conflict,
                "Workflow with {}/{} already exists!",
                workflow_def.name,
                workflow_def.version
            );
        } else {
            Self::insert_or_update_workflow_def(workflow_def);
            Ok(())
        }
    }

    pub fn update_workflow_def(workflow_def: WorkflowDef) {
        Self::insert_or_update_workflow_def(workflow_def);
    }

    fn insert_or_update_workflow_def(workflow_def: WorkflowDef) {
        let workflow_name = workflow_def.name.clone();
        let version = workflow_def.version;

        // First set the workflow def
        WORKFLOW_DEF
            .entry(workflow_name.clone())
            .or_default()
            .insert(version, workflow_def);

        WORKFLOW_DEF_NAMES.insert(workflow_name.clone());
    }

    pub fn get_latest_workflow_def(
        name: &InlineStr,
    ) -> Option<(Ref<InlineStr, HashMap<i32, WorkflowDef>>, &WorkflowDef)> {
        if let Some(max_version) = Self::get_workflow_max_version(name) {
            let guard = WORKFLOW_DEF.get(name);
            if guard.is_none() {
                return None;
            }
            let guard = guard.expect("not none always");

            let workflow_def = guard.get(&max_version);
            if workflow_def.is_none() {
                return None;
            }
            let workflow_def = unsafe {
                (workflow_def.expect("not none always") as *const WorkflowDef)
                    .as_ref()
                    .expect("don't worry")
            };

            return Some((guard, workflow_def));
        }
        None
    }

    fn get_workflow_max_version(workflow_name: &InlineStr) -> Option<i32> {
        WORKFLOW_DEF
            .get(workflow_name)
            .and_then(|x| x.value().keys().max().map(|x| *x))
    }

    #[allow(unused)]
    pub fn get_all_versions(
        name: &InlineStr,
    ) -> Option<(Ref<InlineStr, HashMap<i32, WorkflowDef>>, Vec<&WorkflowDef>)> {
        let guard = WORKFLOW_DEF.get(name);
        if guard.is_none() {
            return None;
        }
        let guard = guard.expect("not none always");

        let workflows = guard
            .value()
            .values()
            .map(|x| unsafe { (x as *const WorkflowDef).as_ref().expect("don't worry") })
            .collect::<Vec<_>>();
        Some((guard, workflows))
    }

    pub fn get_workflow_def(
        name: &InlineStr,
        version: i32,
    ) -> Option<(Ref<InlineStr, HashMap<i32, WorkflowDef>>, &WorkflowDef)> {
        let guard = WORKFLOW_DEF.get(name);
        if guard.is_none() {
            return None;
        }
        let guard = guard.expect("not none always");

        let workflow_def = guard.get(&version);
        if workflow_def.is_none() {
            return None;
        }
        let workflow_def = unsafe {
            (workflow_def.expect("not none always") as *const WorkflowDef)
                .as_ref()
                .expect("don't worry")
        };
        return Some((guard, workflow_def));
    }

    #[allow(unused)]
    pub fn remove_workflow_def(name: &InlineStr, version: i32) -> TegResult<()> {
        if let None = WORKFLOW_DEF
            .get_mut(name)
            .and_then(|mut x| x.remove(&version))
        {
            fmt_err!(
                NotFound,
                "Cannot remove the workflow - no such workflow definition: {} version: {}",
                name,
                version
            )
        } else {
            // check if there are any more versions remaining if not delete the
            // workflow name
            let max_version = Self::get_workflow_max_version(name);

            // delete workflow name
            if max_version.is_none() {
                WORKFLOW_DEF.remove(name);
                WORKFLOW_DEF_NAMES.remove(name);
            }
            Ok(())
        }
    }

    #[allow(unused)]
    pub fn get_all_workflow_defs() -> Vec<SetRefMulti<'static, InlineStr>> {
        // Get all from WORKFLOW_DEF_NAMES
        WORKFLOW_DEF_NAMES.iter().collect::<Vec<_>>()
    }
}
