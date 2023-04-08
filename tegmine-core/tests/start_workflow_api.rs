use std::thread;
use std::time::Duration;

use tegmine_common::StartWorkflowRequest;
use tegmine_core::{ExecutionService, WorkflowService};

#[test]
fn start_workflow() {
    let start_workflow_request = r#"
    {
        "name": "myWorkflow",  
        "version": 1,  
        "correlationId": "corr1",  
        "priority": 1,  
            "input": {  
            "param1": "value1",
            "param2": "value2"
            },
        "taskToDomain": {}
    }"#;
    let start_workflow_request: serde_json::Value =
        serde_json::from_str(start_workflow_request).expect("parse json failed");
    let start_workflow_request: StartWorkflowRequest = start_workflow_request
        .try_into()
        .expect("parse StartWorkflowRequest failed");
    let workflow_instance_id =
        WorkflowService::start_workflow(start_workflow_request).expect("start_workflow failed");
    eprintln!("workflow_instance_id is: {}", workflow_instance_id);

    let mut workflow = ExecutionService::get_execution_status(workflow_instance_id.as_str(), false)
        .expect("get_execution_status failed");
    while !workflow.inner.status.is_terminal() {
        thread::sleep(Duration::from_millis(100));
        workflow = ExecutionService::get_execution_status(workflow_instance_id.as_str(), false)
            .expect("get_execution_status failed");
    }

    if workflow.inner.status.is_successful() {
        eprintln!("workflow execute successful");
        eprintln!("output: {:?}", workflow.inner.output);
    } else {
        assert!(false, "workflow execute failed");
    }
}
