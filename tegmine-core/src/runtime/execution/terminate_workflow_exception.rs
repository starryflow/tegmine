use tegmine_common::prelude::*;

use crate::model::{TaskModel, WorkflowStatus};

thread_local! {
    pub static STATUS: RefCell<Option<WorkflowStatus>> = RefCell::new(None);
    pub static TASK: RefCell<Option<TaskModel>> = RefCell::new(None);
}
