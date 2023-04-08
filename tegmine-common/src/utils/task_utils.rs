use numtoa::NumToA;

use crate::prelude::InlineStr;

pub struct TaskUtils;

impl TaskUtils {
    pub const LOOP_TASK_DELIMITER: &'static str = "__";

    pub fn append_iteration(name: &mut InlineStr, iteration: i32) {
        name.push_str(Self::LOOP_TASK_DELIMITER);
        name.push_str(iteration.numtoa_str(10, &mut [0; 4]))
    }

    pub fn remove_iteration_from_task_ref_name(reference_task_name: &str) -> &str {
        if let Some(pos) = reference_task_name.find(Self::LOOP_TASK_DELIMITER) {
            &reference_task_name[..pos]
        } else {
            reference_task_name
        }
    }
}
