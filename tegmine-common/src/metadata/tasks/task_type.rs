use std::collections::HashSet;
use std::str::FromStr;

use once_cell::sync::Lazy;
use strum_macros::{AsRefStr, EnumString};

#[derive(EnumString, AsRefStr, PartialEq, Eq)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskType {
    SetVariable,
    Switch,
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
    Wait,
    Human,
    Http,
    Inline,
    KafkaPublish,
    JsonJqTransform,

    ExclusiveJoin,
    // #[deprecated]
    // Switch Instead
    // Decision,
    // #[deprecated]
    // Inline Instead
    // Lambda,
}

static BUILT_IN_TASKS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from_iter([
        TaskType::Switch.as_ref(),
        TaskType::Join.as_ref(), // TODO: No ForkJoin ?
        TaskType::ExclusiveJoin.as_ref(),
        TaskType::DoWhile.as_ref(),
        // TaskType::Decision.as_ref(),
        // TaskType::Fork.as_ref(),
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
