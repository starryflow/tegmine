use linked_hash_map::LinkedHashMap;

use crate::metadata::tasks::TaskDef;
use crate::prelude::*;
use crate::TaskType;

/// This is the task definition definied as part of the `WorkflowDef`. The tasks definied in the
/// Workflow definition are saved as part of `WorkflowDef::getTasks`
#[derive(Clone, Debug)]
pub struct WorkflowTask {
    /// Name of the task. MUST be registered as a Task Type with Tegmine before starting workflow
    pub name: InlineStr,
    /// Name of the task. MUST be registered as a Task Type with Tegmine before starting workflow
    pub task_reference_name: InlineStr,
    /// Type of task. SIMPLE for tasks executed by remote workers, or one of the system task types
    pub type_: InlineStr,
    /// Description of the task
    pub description: InlineStr,
    /// true or false. When set to true - workflow continues even if the task fails. The status of
    /// the task is reflected as COMPLETED_WITH_ERRORS
    pub optional: bool,
    /// JSON template that defines the input given to the task. Only one of inputParameters or
    /// inputExpression can be used in a task.
    pub input_parameters: HashMap<InlineStr, Object>,
    /// JSONPath expression that defines the input given to the task. Only one of inputParameters
    /// or inputExpression can be used in a task.
    // pub input_expression
    /// false to mark status Completed upon execution; true to keep the task InProgress and wait
    /// for an external event to complete it.
    pub async_complete: bool,
    /// Time in seconds to wait before making the task available to be polled by a worker.
    pub start_delay: i32,

    /// SWITCH
    /// Type of the evaluator used. Supported types: value-param, javascript.
    pub evaluator_type: InlineStr,
    /// Depends on the evaluatorType:
    /// if `value-param`, Reference to provided key in inputParameters
    /// if `javascript`, Evaluate JavaScript expressions and compute value
    pub expression: InlineStr,
    /// Map where the keys are the possible values that can result from expression being evaluated
    /// by evaluatorType with values being lists of tasks to be executed.
    pub decision_cases: LinkedHashMap<InlineStr, Vec<WorkflowTask>>,
    /// List of tasks to be executed when no matching value if found in decisionCases
    pub default_case: Vec<WorkflowTask>,

    /// EXCLUSIVE_JOIN
    pub exclusive_join_on: Vec<InlineStr>,
    pub default_exclusive_join_task: Vec<InlineStr>,

    /// DYNAMIC
    /// Name of the parameter from inputParameters whose value is used to schedule the task. e.g.
    /// "taskToExecute"
    pub dynamic_task_name_param: InlineStr,
    /// DO_WHILE
    /// Condition to be evaluated after every iteration. This is a Rhai expression, evaluated
    /// using the rhai engine. If an exception occurs during evaluation, the DO_WHILE task is
    /// set to FAILED_WITH_TERMINAL_ERROR.
    pub loop_condition: InlineStr,
    /// List of tasks that needs to be executed as long as the condition is true.
    pub loop_over: Vec<WorkflowTask>,

    // /// SUB_WORKFLOW
    // pub sub_workflow_param: Option<SubWorkflowParams>,

    // /// FORK_JOIN/JOIN
    pub fork_tasks: Vec<Vec<WorkflowTask>>,
    pub join_on: Vec<InlineStr>,

    // /// FORK_JOIN_DYNAMIC
    // pub dynamic_fork_tasks_param: InlineStr,
    // pub dynamic_fork_tasks_input_param_name: InlineStr,

    // /// Event
    // pub sink: InlineStr,

    // ///
    // pub rate_limited: bool,
    pub retry_count: i32,
    pub task_definition: Option<TaskDef>,
    // /// deprecated
    // pub case_value_param: InlineStr,
    // /// deprecated
    // pub case_expression: InlineStr,
    // /// deprecated
    // pub script_expression: InlineStr,
    // /// deprecated
    // pub dynamic_fork_join_tasks_param: InlineStr,
}

impl WorkflowTask {
    fn children(&self) -> Vec<&Vec<WorkflowTask>> {
        let mut workflow_task_lists = Vec::default();
        match TaskType::of(self.type_.as_str()) {
            TaskType::Switch => {
                workflow_task_lists.extend(self.decision_cases.values());
                workflow_task_lists.push(&self.default_case);
            }
            // TaskType::ForkJoin => workflow_task_lists.extend(&self.fork_tasks),
            TaskType::DoWhile => workflow_task_lists.push(&self.loop_over),
            _ => {}
        }
        workflow_task_lists
    }

    fn children_mut(&mut self) -> Vec<&mut Vec<WorkflowTask>> {
        let mut workflow_task_lists = Vec::default();
        match TaskType::of(self.type_.as_str()) {
            TaskType::Switch => {
                for (_, tasks) in self.decision_cases.iter_mut() {
                    workflow_task_lists.push(tasks);
                }
                workflow_task_lists.push(&mut self.default_case);
            }
            // TaskType::ForkJoin => workflow_task_lists.extend(&mut self.fork_tasks),
            TaskType::DoWhile => workflow_task_lists.push(&mut self.loop_over),
            _ => {}
        }
        workflow_task_lists
    }

    pub fn collect_tasks(&self) -> Vec<&WorkflowTask> {
        let mut tasks = Vec::default();
        tasks.push(self);

        for workflow_task_list in self.children() {
            for workflow_task in workflow_task_list {
                tasks.extend(workflow_task.collect_tasks())
            }
        }
        tasks
    }

    pub fn populate_tasks(&mut self, populate_fn: fn(&mut WorkflowTask)) {
        populate_fn(self);

        for workflow_task_list in self.children_mut() {
            for workflow_task in workflow_task_list {
                populate_fn(workflow_task);
            }
        }
    }

    pub fn next<'a>(
        &'a self,
        task_reference_name: &str,
        parent: Option<&'a WorkflowTask>,
    ) -> Option<&WorkflowTask> {
        let task_type = TaskType::of(self.type_.as_str());
        match task_type {
            TaskType::DoWhile | TaskType::Switch => {
                for workflow_tasks in self.children() {
                    let mut iterator = workflow_tasks.iter();
                    while let Some(task) = iterator.next() {
                        if task.task_reference_name.eq(task_reference_name) {
                            break;
                        }
                        if let Some(next_task) = task.next(task_reference_name, Some(self)) {
                            return Some(next_task);
                        }
                        if task.has(task_reference_name) {
                            break;
                        }
                    }
                    if let Some(next_task) = iterator.next() {
                        return Some(next_task);
                    }
                }
                if task_type == TaskType::DoWhile && self.has(task_reference_name) {
                    // come here means this is DO_WHILE task and `taskReferenceName` is the last
                    // task in
                    // this DO_WHILE task, because DO_WHILE task need to be executed to decide
                    // whether to
                    // schedule next iteration, so we just return the DO_WHILE task, and then ignore
                    // generating this task again in deciderService.getNextTask()
                    return Some(self);
                }
            }
            TaskType::ForkJoin => {
                let mut found = false;
                for workflow_tasks in self.children() {
                    let mut iterator = workflow_tasks.iter();
                    while let Some(task) = iterator.next() {
                        if task.task_reference_name.eq(task_reference_name) {
                            found = true;
                            break;
                        }
                        if let Some(next_task) = task.next(task_reference_name, Some(self)) {
                            return Some(next_task);
                        }
                        if task.has(task_reference_name) {
                            break;
                        }
                    }
                    if let Some(next_task) = iterator.next() {
                        return Some(next_task);
                    }
                    if found && parent.is_some() {
                        // we need to return join task... -- get my sibling from my parent..
                        return parent
                            .expect("not none")
                            .next(&self.task_reference_name, parent);
                    }
                }
            }
            _ => {}
        }
        None
    }

    pub fn has(&self, task_reference_name: &str) -> bool {
        if self.task_reference_name.eq(task_reference_name) {
            return true;
        }

        match TaskType::of(self.type_.as_str()) {
            TaskType::Switch | TaskType::DoWhile | TaskType::ForkJoin => {
                for child_x in self.children() {
                    for child in child_x {
                        if child.has(task_reference_name) {
                            return true;
                        }
                    }
                }
            }
            _ => {}
        }
        false
    }
}

impl TryFrom<&serde_json::Value> for WorkflowTask {
    type Error = ErrorCode;
    fn try_from(value: &serde_json::Value) -> Result<Self, Self::Error> {
        let type_: InlineStr = value
            .get("type")
            .and_then(|x| x.as_str())
            .ok_or_else(|| ErrorCode::IllegalArgument("WorkflowTask: type not found"))?
            .trim()
            .into();

        // inputParameters
        let input_parameters = Object::convert_jsonmap_to_hashmap(
            value
                .get("inputParameters")
                .and_then(|x| x.as_object())
                .ok_or_else(|| {
                    ErrorCode::IllegalArgument("WorkflowTask: inputParameters invalid")
                })?,
        );

        // SWITCH
        let (evaluator_type, expression, decision_cases, default_case) =
            Self::switch_try_from(&type_, value)?;

        // DYNAMIC
        let dynamic_task_name_param = Self::dynamic_try_from(&type_, &input_parameters, value)?;

        // SET_VARIABLE
        if type_.eq("SET_VARIABLE") && input_parameters.is_empty() {
            return fmt_err!(
                IllegalArgument,
                "WorkflowTask: inputParameters can not be empty when task type is SET_VARIABLE"
            );
        }

        // TERMINATE
        if type_.eq("TERMINATE") {
            if !input_parameters.contains_key("terminationStatus") {
                return fmt_err!(
                    IllegalArgument,
                    "WorkflowTask: terminationStatus in inputParameters not found when task type is TERMINATE"
                );
            }
        }

        // DO_WHILE
        let (loop_condition, loop_over) = Self::loop_try_from(&type_, value)?;

        // FORK_JOIN
        let fork_tasks = Self::fork_join_try_from(&type_, value)?;

        // JOIN
        let join_on = Self::join_try_from(&type_, value, &fork_tasks)?;

        // EXCLUSIVE_JOIN
        let (exclusive_join_on, default_exclusive_join_task) =
            Self::exclusive_join_try_from(&type_, value)?;

        // TODO
        {
            // START_WORKFLOW -> inputParameters<startWorkflow>
            // SUB_WORKFLOW -> subWorkflowParam

            // FORK_JOIN_DYNAMIC -> dynamicForkTasksParam, dynamicForkTasksInputParamName,
            // inputParameters<dynamicTasks, dynamicTasksInput>
            //
            // EVENT -> sink, asyncComplete
            // HTTP -> inputParameters<http_request>
            // INLINE -> inputParameters<evaluatorType, expression>
            // JSON_JQ_TRANSFORM -> inputParameters<queryExpression>
            // KAFKA_PUBLISH -> inputParameters<kafka_request, ...>
            // WAIT -> inputParameters<duration or until>
        }

        // ad hoc TaskDef
        let task_definition = if let Some(task_def) = value.get("taskDefinition") {
            Some(TaskDef::try_from(task_def)?)
        } else {
            None
        };

        Ok(Self {
            name: value
                .get("name")
                .and_then(|x| x.as_str())
                .ok_or_else(|| ErrorCode::IllegalArgument("WorkflowTask: name not found"))?
                .trim()
                .into(),
            task_reference_name: value
                .get("taskReferenceName")
                .and_then(|x| x.as_str())
                .ok_or_else(|| {
                    ErrorCode::IllegalArgument("WorkflowTask: taskReferenceName not found")
                })?
                .trim()
                .into(),
            type_,
            description: value
                .get("description")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .trim()
                .into(),
            optional: value
                .get("optional")
                .unwrap_or(&serde_json::json!(false))
                .as_bool()
                .ok_or_else(|| ErrorCode::IllegalArgument("WorkflowTask: optional invalid"))?,
            input_parameters,
            async_complete: value
                .get("asyncComplete")
                .unwrap_or(&serde_json::json!(false))
                .as_bool()
                .ok_or_else(|| ErrorCode::IllegalArgument("WorkflowTask: asyncComplete invalid"))?,
            start_delay: value
                .get("startDelay")
                .unwrap_or(&serde_json::json!(0))
                .as_i64()
                .ok_or_else(|| ErrorCode::IllegalArgument("WorkflowTask: startDelay invalid"))?
                as i32,
            evaluator_type,
            expression,
            decision_cases,
            exclusive_join_on,
            default_exclusive_join_task,
            default_case,
            dynamic_task_name_param,
            loop_condition,
            loop_over,
            // sub_workflow_param: (),
            fork_tasks,
            join_on,
            // dynamic_fork_tasks_param: (),
            // dynamic_fork_tasks_input_param_name: (),
            // sink: (),
            // rate_limited: (),
            retry_count: 0,
            task_definition,
            // case_value_param: (),
            // case_expression: (),
            // script_expression: (),
            // dynamic_fork_join_tasks_param: (),
        })
    }
}

impl WorkflowTask {
    pub fn try_from_jsonlist(jsonlist: &Vec<serde_json::Value>) -> TegResult<Vec<Self>> {
        let mut tasks = Vec::with_capacity(jsonlist.len());
        for json in jsonlist {
            tasks.push(json.try_into()?);
        }
        Ok(tasks)
    }

    fn switch_try_from(
        type_: &InlineStr,
        value: &serde_json::Value,
    ) -> TegResult<(
        InlineStr,
        InlineStr,
        LinkedHashMap<InlineStr, Vec<WorkflowTask>>,
        Vec<WorkflowTask>,
    )> {
        if type_.eq("SWITCH") {
            let evaluator_type: InlineStr = value
                .get("evaluatorType")
                .and_then(|x| x.as_str())
                .ok_or_else(|| ErrorCode::IllegalArgument("WorkflowTask: evaluatorType not found"))?
                .trim()
                .into();
            if !evaluator_type.as_str().eq("value-param")
                && !evaluator_type.as_str().eq("javascript")
            {
                return fmt_err!(
                    IllegalArgument,
                    "WorkflowTask: evaluatorType invalid, not ''value-param' or 'javascript'"
                );
            }

            let expression: InlineStr = value
                .get("expression")
                .and_then(|x| x.as_str())
                .ok_or_else(|| ErrorCode::IllegalArgument("WorkflowTask: expression not found"))?
                .trim()
                .into();

            let decision_cases = WorkflowTask::decision_case_try_from(
                value
                    .get("decisionCases")
                    .and_then(|x| x.as_object())
                    .ok_or_else(|| {
                        ErrorCode::IllegalArgument("WorkflowTask: decisionCases invalid")
                    })?,
            )?;
            if decision_cases.is_empty() {
                return fmt_err!(
                    IllegalArgument,
                    "WorkflowTask: decisionCases can not be empty"
                );
            }

            let default_case = WorkflowTask::try_from_jsonlist(
                value
                    .get("defaultCase")
                    .and_then(|x| x.as_array())
                    .ok_or_else(|| {
                        ErrorCode::IllegalArgument("WorkflowTask: defaultCase invalid")
                    })?,
            )?;
            if default_case.is_empty() {
                return fmt_err!(
                    IllegalArgument,
                    "WorkflowTask: defaultCase can not be empty"
                );
            }
            Ok((evaluator_type, expression, decision_cases, default_case))
        } else {
            Ok((
                InlineStr::default(),
                InlineStr::default(),
                LinkedHashMap::default(),
                Vec::default(),
            ))
        }
    }

    fn decision_case_try_from(
        jsonmap: &serde_json::Map<String, serde_json::Value>,
    ) -> TegResult<LinkedHashMap<InlineStr, Vec<Self>>> {
        let mut tasks = LinkedHashMap::with_capacity(jsonmap.len());
        for (k, v) in jsonmap {
            let jsonlist = v
                .as_array()
                .ok_or_else(|| ErrorCode::IllegalArgument("WorkflowTask: decisionCases invalid"))?;
            tasks.insert(k.into(), Self::try_from_jsonlist(jsonlist)?);
        }
        Ok(tasks)
    }

    fn dynamic_try_from(
        type_: &InlineStr,
        input_parameters: &HashMap<InlineStr, Object>,
        value: &serde_json::Value,
    ) -> TegResult<InlineStr> {
        if type_.eq("DYNAMIC") {
            let dynamic_task_name_param = value
                .get("dynamicTaskNameParam")
                .and_then(|x| x.as_str())
                .ok_or_else(|| {
                    ErrorCode::IllegalArgument("WorkflowTask: dynamicTaskNameParam not found")
                })?
                .trim()
                .into();
            if !input_parameters.contains_key(&dynamic_task_name_param) {
                return fmt_err!(
                    IllegalArgument,
                    "WorkflowTask: dynamicTaskNameParam invalid: can not find {} in inputParameters",
                    dynamic_task_name_param
                );
            }
            Ok(dynamic_task_name_param)
        } else {
            Ok(InlineStr::default())
        }
    }

    fn loop_try_from(
        type_: &InlineStr,
        value: &serde_json::Value,
    ) -> TegResult<(InlineStr, Vec<WorkflowTask>)> {
        if type_.eq("DO_WHILE") {
            let loop_condition = value
                .get("loopCondition")
                .and_then(|x| x.as_str())
                .ok_or_else(|| ErrorCode::IllegalArgument("WorkflowTask: loopCondition not found"))?
                .trim()
                .into();
            let loop_over = WorkflowTask::try_from_jsonlist(
                value
                    .get("loopOver")
                    .and_then(|x| x.as_array())
                    .ok_or_else(|| ErrorCode::IllegalArgument("WorkflowTask: loopOver invalid"))?,
            )?;
            if loop_over.is_empty() {
                return fmt_err!(IllegalArgument, "WorkflowTask: loopOver can not be empty");
            }
            Ok((loop_condition, loop_over))
        } else {
            Ok((InlineStr::default(), Vec::default()))
        }
    }

    fn fork_join_try_from(
        type_: &InlineStr,
        value: &serde_json::Value,
    ) -> TegResult<Vec<Vec<WorkflowTask>>> {
        if type_.eq("FORK_JOIN") {
            let mut fork_tasks = Vec::default();
            for array in value
                .get("forkTasks")
                .and_then(|x| x.as_array())
                .ok_or_else(|| ErrorCode::IllegalArgument("WorkflowTask: fork_join invalid"))?
            {
                let tasks =
                    WorkflowTask::try_from_jsonlist(array.as_array().ok_or_else(|| {
                        ErrorCode::IllegalArgument("WorkflowTask: fork_join invalid")
                    })?)?;
                if tasks.is_empty() {
                    return fmt_err!(
                        IllegalArgument,
                        "WorkflowTask: tasks in forkTasks can not be empty"
                    );
                }
                fork_tasks.push(tasks);
            }

            if fork_tasks.is_empty() {
                return fmt_err!(IllegalArgument, "WorkflowTask: forkTasks can not be empty");
            }
            Ok(fork_tasks)
        } else {
            Ok(Vec::default())
        }
    }

    fn join_try_from(
        type_: &InlineStr,
        value: &serde_json::Value,
        fork_tasks: &Vec<Vec<WorkflowTask>>,
    ) -> TegResult<Vec<InlineStr>> {
        if type_.eq("JOIN") {
            let mut join_on = Vec::default();

            for v in value
                .get("joinOn")
                .and_then(|x| x.as_array())
                .ok_or_else(|| ErrorCode::IllegalArgument("WorkflowTask: join invalid"))?
            {
                let fork_task_ref = v
                    .as_str()
                    .ok_or_else(|| ErrorCode::IllegalArgument("WorkflowTask: joinOn invalid"))?
                    .into();
                let mut exist_ref = false;
                for tasks in fork_tasks {
                    if tasks
                        .iter()
                        .find(|x| x.task_reference_name.eq(&fork_task_ref))
                        .is_some()
                    {
                        exist_ref = true;
                        break;
                    }
                }
                if !exist_ref {
                    return fmt_err!(IllegalArgument, "WorkflowTask: joinOn task_ref not exist");
                }
                join_on.push(fork_task_ref);
            }
            if join_on.is_empty() {
                return fmt_err!(IllegalArgument, "WorkflowTask: joinOn can not be empty");
            }
            Ok(join_on)
        } else {
            Ok(Vec::default())
        }
    }

    fn exclusive_join_try_from(
        type_: &InlineStr,
        value: &serde_json::Value,
    ) -> TegResult<(Vec<InlineStr>, Vec<InlineStr>)> {
        if type_.eq("EXCLUSIVE_JOIN") {
            let mut exclusive_join_on = Vec::default();

            for v in value
                .get("exclusiveJoinOn")
                .and_then(|x| x.as_array())
                .ok_or_else(|| ErrorCode::IllegalArgument("WorkflowTask: exclusive_join invalid"))?
            {
                let fork_task_ref = v
                    .as_str()
                    .ok_or_else(|| {
                        ErrorCode::IllegalArgument("WorkflowTask: exclusive_join invalid")
                    })?
                    .into();
                exclusive_join_on.push(fork_task_ref);
            }
            if exclusive_join_on.is_empty() {
                return fmt_err!(
                    IllegalArgument,
                    "WorkflowTask: joinOn in exclusive_join can not be empty"
                );
            }

            let mut default_exclusive_join_task = Vec::default();
            for v in value
                .get("defaultExclusiveJoinTask")
                .and_then(|x| x.as_array())
                .ok_or_else(|| ErrorCode::IllegalArgument("WorkflowTask: exclusive_join invalid"))?
            {
                let default_exclusive_join_task_ref = v
                    .as_str()
                    .ok_or_else(|| {
                        ErrorCode::IllegalArgument("WorkflowTask: exclusive_join invalid")
                    })?
                    .into();
                default_exclusive_join_task.push(default_exclusive_join_task_ref);
            }
            if default_exclusive_join_task.is_empty() {
                return fmt_err!(
                    IllegalArgument,
                    "WorkflowTask: defaultExclusiveJoinTask in exclusive_join can not be empty"
                );
            }

            Ok((exclusive_join_on, default_exclusive_join_task))
        } else {
            Ok((Vec::default(), Vec::default()))
        }
    }
}
