use std::collections::HashSet;
use std::str::FromStr;

use once_cell::sync::Lazy;
use strum_macros::{AsRefStr, EnumString};

#[derive(EnumString, AsRefStr, PartialEq, Eq)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskType {
    SetVariable,
    Switch,
    ExclusiveJoin, // see https://github.com/Netflix/conductor/issues/2759
    Dynamic,
    ForkJoin,
    Join,
    ForkJoinDynamic,
    UserDefined,
    DoWhile,
    Simple,
    StartWorkflow,
    SubWorkflow,
    Terminate,

    Event,
    Human,
    Http,
    Inline,
    JsonJqTransform,
    KafkaPublish,
    Wait,
    // deprecated: Switch Instead
    // Decision,
    // deprecated: Inline Instead
    // Lambda,
}

static BUILT_IN_TASKS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from_iter([
        TaskType::Switch.as_ref(),
        TaskType::ExclusiveJoin.as_ref(),
        TaskType::TASK_TYPE_FORK, // see: ForkJoinDynamicTaskMapper
        TaskType::Join.as_ref(),
        TaskType::DoWhile.as_ref(),
    ])
});

impl TaskType {
    pub const TASK_TYPE_FORK: &'static str = "FORK";

    /// Converts a task type string to `TaskType`. For an unknown string, the value is defaulted to
    /// `TaskType::UserDefined`.
    pub fn of(task_type: &str) -> TaskType {
        TaskType::from_str(task_type).unwrap_or(TaskType::UserDefined)
    }

    pub fn is_builtin(task_type: &str) -> bool {
        BUILT_IN_TASKS.contains(task_type)
    }
}
