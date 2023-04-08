use std::collections::HashSet;
use std::str::FromStr;

use once_cell::sync::Lazy;
use strum_macros::{AsRefStr, EnumString};

#[derive(EnumString, AsRefStr, PartialEq, Eq)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskType {
    Simple,
    Dynamic,
    ForkJoin,
    ForkJoinDynamic,
    Decision,
    Switch,
    Join,
    DoWhile,
    SubWorkflow,
    StartWorkflow,
    Event,
    Wait,
    Human,
    UserDefined,
    Http,
    Lambda,
    Inline,
    ExclusiveJoin,
    Terminate,
    KafkaPublish,
    JsonJqTransform,
    SetVariable,
}

static BUILT_IN_TASKS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from_iter([
        TaskType::Decision.as_ref(),
        TaskType::Switch.as_ref(),
        // TaskType::Fork.as_ref(),
        TaskType::Join.as_ref(),
        TaskType::ExclusiveJoin.as_ref(),
        TaskType::DoWhile.as_ref(),
    ])
});

impl TaskType {
    /// Converts a task type string to `TaskType`. For an unknown string, the value is defaulted to
    /// `TaskType::USER_DEFINED`.
    pub fn of(task_type: &str) -> TaskType {
        TaskType::from_str(task_type).unwrap_or(TaskType::UserDefined)
    }

    pub fn is_builtin(task_type: &str) -> bool {
        BUILT_IN_TASKS.contains(task_type)
    }
}
