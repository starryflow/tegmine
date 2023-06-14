use tegmine_common::prelude::*;

use crate::model::TaskModel;

pub struct QueueUtils;

impl QueueUtils {
    pub const DOMAIN_SEPARATOR: &str = ":";
    pub const ISOLATION_SEPARATOR: &str = "-";
    pub const EXECUTION_NAME_SPACE_SEPARATOR: &str = "@";

    pub fn get_queue_name_by_task_model(task_model: &TaskModel) -> InlineStr {
        Self::get_queue_name(
            &task_model.task_type,
            &task_model.domain,
            &task_model.isolation_group_id,
            &task_model.execution_name_space,
        )
    }

    pub fn get_queue_name(
        task_type: &InlineStr,
        domain: &InlineStr,
        isolation_group_id: &InlineStr,
        execution_name_space: &InlineStr,
    ) -> InlineStr {
        let mut queue_name = if domain.is_empty() {
            task_type.clone()
        } else {
            let mut queue_name = domain.clone();
            queue_name.push_str(Self::DOMAIN_SEPARATOR);
            queue_name.push_str(task_type.as_str());
            queue_name
        };

        if !execution_name_space.is_empty() {
            queue_name.push_str(Self::EXECUTION_NAME_SPACE_SEPARATOR);
            queue_name.push_str(execution_name_space.as_str());
        }

        if !isolation_group_id.is_empty() {
            queue_name.push_str(Self::ISOLATION_SEPARATOR);
            queue_name.push_str(isolation_group_id.as_str());
        }

        queue_name
    }

    pub fn get_task_type(queue: &InlineStr) -> InlineStr {
        if queue.is_empty() {
            return InlineStr::from("");
        }

        let domain_separator_index = queue.find(Self::DOMAIN_SEPARATOR);
        let start_index = match domain_separator_index {
            Some(index) => index + 1,
            None => 0,
        };

        let end_index = queue
            .find(Self::EXECUTION_NAME_SPACE_SEPARATOR)
            .or_else(|| queue.rfind(Self::ISOLATION_SEPARATOR))
            .unwrap_or(queue.len());

        InlineStr::from(&queue.as_str()[start_index..end_index])
    }
}
